// Rooms through an `og relay` over the browser's WebSocket: the web room
// provider for a page that is not the Player's (docs/web-export.md
// §Providers). It speaks og-net's protocol (crates/og-net/src/protocol.rs)
// as the native client does (crates/og-net/src/client.rs, `og run --room`):
//
// - One connection, opened when the provider is made. A URL with a room in
//   its path (`ws://127.0.0.1:47900/arena`) joins that room as it opens;
//   one to `/` sends `create`, `join` or `quick` itself. A page with a Play
//   link (`?room=<id>`) needs no path: the engine joins the link's room.
// - Every op is one text frame, the engine's op as it is, in order; ops
//   made while the socket opens wait and go first, in order. Every text
//   frame from the relay is one event, in order; a frame that is not JSON,
//   and any binary frame, is ignored. The size limits (64 KiB a game's
//   message, 16 MiB an engine channel's) are the relay's: it answers a
//   larger `send` with an `error` `too_large`.
// - A lost connection is mended, as natively (the protocol's contract): the
//   provider connects again, first after 250 ms and then after twice as
//   long each time up to 4 s, for `reconnectMs` (30 s). Back, it asks again
//   what was unanswered, or joins the room it was in again under the name
//   it had, naming the number it had (`was`, and a quick play room's key):
//   the engine hears `joined` once more, with this peer's new number, and
//   the other peers `peer_left` and a `peer_joined` that carries `was`.
//   What was sent meanwhile is lost. Only when the window
//   passes, or the room refuses it, is the room a `left` (`the relay closed
//   the connection`, `the relay connection failed`); an ask that waited
//   hears an `error` `offline`. Each later create, join or quick tries the
//   relay once more and hears `offline` when it is still away, as the asks
//   that waited for a connection that never opened do, so a join never
//   waits for nothing.
// - A browser tells a page of a lost connection when its system does; it
//   has no ping of its own to send. The relay asks a silent connection
//   itself and drops one that does not answer in 10 s, so this peer's place
//   in the room is free by the time the page is told and comes back.
// - close() sends what is queued and then closes: the other peers hear this
//   one's last messages before its `peer_left`.
//
// A page served over https reaches only a `wss://` relay (a browser refuses
// `ws://` from a secure page, except to localhost); `og relay` itself serves
// plain `ws://`, so a public one sits behind a TLS proxy.

import type { RoomProvider } from "../provider.ts";

/** The web relay room provider, and what its page may do with it. */
export interface RelayRooms extends RoomProvider {
  readonly host: "relay";
  readonly url: string;
  /** `connecting` (also while a lost connection is tried again), `open` or `closed`. */
  readonly state: "connecting" | "open" | "closed";
  /** Send what is queued, then close the connection. */
  close(): void;
}

/**
 * The part of the WebSocket constructor this needs (the browser's by default), written out
 * rather than named from the DOM: a game's type check has no DOM, and og.platform's provider
 * module, which re-exports relayRooms, is part of it.
 */
export type SocketFactory = new (url: string) => {
  send(data: string): void;
  close(code?: number): void;
  readonly readyState: number;
  onopen: ((e: unknown) => unknown) | null;
  onmessage: ((e: { readonly data: unknown }) => unknown) | null;
  onclose: ((e: { readonly wasClean: boolean }) => unknown) | null;
  onerror: ((e: unknown) => unknown) | null;
};

/** The ops that ask for a room: a lost or unreachable relay answers each with an `error`. */
const ASKS = new Set(["create", "join", "quick"]);

/** The protocol's reconnect waits (crates/og-net/src/protocol.rs). */
const RECONNECT_FIRST_MS = 250;
const RECONNECT_MAX_MS = 4000;
const RECONNECT_WINDOW_MS = 30_000;

type Op = Record<string, unknown>;

/** The page's timers, written out as the socket is: a game's type check has no DOM. */
const timers = globalThis as unknown as { setTimeout(f: () => void, ms: number): unknown; clearTimeout(timer: unknown): void };

/**
 * Rooms through the `og relay` at `url` (`ws://host:port/` or `wss://host/`, optionally with a
 * room in the path). Throws for a URL that is not a WebSocket URL, as `og run --room` refuses it.
 * `reconnectMs` is how long a lost connection is tried again before the room is lost (30 s; 0 never tries).
 */
export function relayRooms(url: string, options: { readonly WebSocket?: SocketFactory; readonly reconnectMs?: number } = {}): RelayRooms {
  const parts = /^(wss?:\/\/[^/]+)\/*(.*?)\/*$/.exec(url);
  if (!parts) throw new Error(`'${url}' is not a relay URL: write ws://127.0.0.1:<port>/ (og relay prints its port) or wss://<host>/ for one behind TLS`);
  const Socket = options.WebSocket ?? (globalThis as unknown as { WebSocket?: SocketFactory }).WebSocket;
  if (!Socket) throw new Error("this page has no WebSocket: relayRooms needs a browser (or a runtime with WebSocket)");
  const window = options.reconnectMs ?? RECONNECT_WINDOW_MS;
  /** The relay without the room: a new connection asks for its room itself, under the name this peer had. */
  const base = `${parts[1]}/`;
  let state: RelayRooms["state"] = "connecting";
  let closing = false;
  let deliver: ((events: unknown[]) => void) | null = null;
  /** Events heard before the page gave a deliver (a room joined by the URL's path). */
  const early: unknown[] = [];
  /** Ops made while a socket opens that nothing is asked of again (the first, or one a later ask opened). */
  const waiting: Op[] = [];
  let socket: InstanceType<SocketFactory> | null = null;
  /** The create, join or quick the relay has not answered yet (the path's room: the relay joined it as the socket opened). */
  let asked: Op | null = /^[A-Za-z0-9._-]{1,64}$/.test(parts[2]!) ? { op: "join", room: parts[2], name: "" } : null;
  /** The room this peer is in, its number and name there and a quick play room's key. */
  let room: { id: string; peer: number; name: string; key?: string } | null = null;
  /** A leave is on its way: `left` answers. */
  let leaving = false;
  /** On a new connection, the join of `room` is on its way: until its `joined`, nothing is sent to it. */
  let rejoining = false;
  /** While a lost connection is tried again: when that ends, the next wait and its timer. */
  let mending: { until: number; wait: number; why: string; timer?: unknown } | null = null;

  const emit = (events: unknown[]) => {
    if (events.length === 0) return;
    if (deliver) deliver(events);
    else early.push(...events);
  };
  const offline = (op: Op, message: string) => ({ type: "error", op: String(op["op"]), code: "offline", message });
  const refuse = (ops: readonly Op[], message: string) => emit(ops.filter((o) => ASKS.has(String(o["op"]))).map((o) => offline(o, message)));
  /** This peer is in no room and asks for none any more: the engine hears `left` if it was in one or waiting for one. */
  const gone = (why: string) => {
    const was = room !== null || asked !== null;
    room = asked = null;
    leaving = rejoining = false;
    if (was) emit([{ type: "left", reason: why }]);
  };
  /** An op while connected: noted, and sent unless it is for a room this peer may not be in. */
  const send = (to: InstanceType<SocketFactory>, op: Op) => {
    const kind = String(op["op"]);
    if (ASKS.has(kind)) {
      if (!room) asked = op;
    } else if (kind === "leave") leaving = true;
    else if (rejoining) return;
    to.send(JSON.stringify(op));
  };
  /** An op while the lost connection is tried again. */
  const away = (op: Op) => {
    const kind = String(op["op"]);
    if (ASKS.has(kind)) {
      if (room) emit([offline(op, `the relay at ${base} is being reached again`)]);
      else asked = op;
    } else if (kind === "leave") gone("left");
  };
  /** An event from the relay: noted, and passed on to the engine. */
  const hear = (event: unknown) => {
    const e = (event ?? {}) as Record<string, unknown>;
    if (e["type"] === "joined") {
      const name = asked ? String(asked["name"] ?? "") : (room?.name ?? "");
      room = { id: String(e["room"] ?? ""), peer: Number(e["peer"] ?? 0), name, ...(typeof e["key"] === "string" ? { key: e["key"] } : {}) };
      asked = null;
      rejoining = false;
    } else if (e["type"] === "left") {
      room = asked = null;
      leaving = rejoining = false;
    } else if (e["type"] === "error" && ASKS.has(String(e["op"]))) {
      // The engine is in the room still, as far as it knows: a refusal to take it back is the room lost.
      if (rejoining) return gone(`the relay connection was lost, and room '${room?.id}' did not take this peer back (${String(e["code"])}: ${String(e["message"])})`);
      if (!room) asked = null;
    }
    emit([event]);
  };

  /** Opens a socket to `target`. `fresh`: nothing is asked of it again (what waits goes as it is), else it stands for the lost one. */
  const connect = (target: string, fresh: boolean) => {
    const s = new Socket(target);
    socket = s;
    let opened = false;
    let ended = false;
    s.onopen = () => {
      opened = true;
      state = "open";
      if (fresh) for (const op of waiting.splice(0)) send(s, op);
      else {
        mending = null;
        // The leave may not have reached the relay; the connection's end was a departure either way.
        if (leaving) gone("left");
        else if (room) {
          rejoining = true;
          s.send(JSON.stringify({ op: "join", room: room.id, name: room.name, was: room.peer, ...(room.key === undefined ? {} : { key: room.key }) }));
        } else if (asked) s.send(JSON.stringify(asked));
      }
      if (closing) finish();
    };
    s.onmessage = (e) => {
      if (typeof e.data !== "string") return;
      let event: unknown;
      try {
        event = JSON.parse(e.data);
      } catch {
        return;
      }
      hear(event);
    };
    const lost = (why: string) => {
      if (ended) return;
      ended = true;
      if (closing || state === "closed") return void (state = "closed");
      if (!opened && !mending) {
        // A connection that never opened: what waited for it is answered, and nothing is tried again.
        state = "closed";
        asked = null;
        return refuse(waiting.splice(0), `cannot reach the relay at ${url}; start one with \`og relay\``);
      }
      mending ??= { until: Date.now() + window, wait: RECONNECT_FIRST_MS, why };
      const m = mending;
      if (Date.now() + m.wait > m.until) {
        mending = null;
        state = "closed";
        if (asked && !room) emit([offline(asked, `the relay at ${base} cannot be reached (${m.why})`)]);
        asked = room ? asked : null;
        return gone(m.why);
      }
      state = "connecting";
      m.timer = timers.setTimeout(() => connect(base, false), m.wait);
      m.wait = Math.min(m.wait * 2, RECONNECT_MAX_MS);
    };
    s.onclose = (e) => lost(e.wasClean ? "the relay closed the connection" : "the relay connection failed");
    s.onerror = () => lost("the relay connection failed");
  };
  const finish = () => {
    state = "closed";
    try {
      socket?.close(1000);
    } catch {
      /* already closing */
    }
  };
  connect(url, true);

  return {
    host: "relay",
    url,
    get state() {
      return state;
    },
    attach(to) {
      deliver = to;
      if (early.length > 0) to(early.splice(0));
    },
    handle(ops, to) {
      deliver = to;
      if (early.length > 0) to(early.splice(0));
      if (closing) return refuse(ops, `the relay connection at ${url} is closed`);
      if (mending) return ops.forEach(away);
      if (state === "closed") {
        // Each ask tries the relay once more; what follows it waits for that socket.
        const first = ops.findIndex((o) => ASKS.has(String(o["op"])));
        if (first < 0) return;
        waiting.push(...ops.slice(first));
        state = "connecting";
        return connect(base, true);
      }
      if (state === "connecting") return void waiting.push(...ops);
      for (const op of ops) send(socket!, op);
    },
    close() {
      if (closing || state === "closed") return;
      closing = true;
      if (mending) {
        timers.clearTimeout(mending.timer);
        mending = null;
        return finish();
      }
      // While connecting: the waiting ops go when it opens, and then it closes.
      if (state === "open") finish();
    },
  };
}
