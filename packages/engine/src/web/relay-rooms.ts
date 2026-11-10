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
// - No reconnect, as natively: a lost connection is a `left` event (`the
//   relay closed the connection`, or `the relay connection failed`), and the
//   room is gone. Where the native client then drops ops unheard, this one
//   answers a later create, join or quick with an `error` `offline`, and a
//   connection that never opened answers the ops that waited for it so, so
//   a join never waits for nothing.
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
  /** `connecting`, `open` or `closed`. */
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

/**
 * Rooms through the `og relay` at `url` (`ws://host:port/` or `wss://host/`, optionally with a
 * room in the path). Throws for a URL that is not a WebSocket URL, as `og run --room` refuses it.
 */
export function relayRooms(url: string, options: { readonly WebSocket?: SocketFactory } = {}): RelayRooms {
  if (!/^wss?:\/\/[^/]/.test(url)) throw new Error(`'${url}' is not a relay URL: write ws://127.0.0.1:<port>/ (og relay prints its port) or wss://<host>/ for one behind TLS`);
  const Socket = options.WebSocket ?? (globalThis as unknown as { WebSocket?: SocketFactory }).WebSocket;
  if (!Socket) throw new Error("this page has no WebSocket: relayRooms needs a browser (or a runtime with WebSocket)");
  let state: RelayRooms["state"] = "connecting";
  let opened = false;
  let closing = false;
  let deliver: ((events: unknown[]) => void) | null = null;
  /** Events heard before the page gave a deliver (a room joined by the URL's path). */
  const early: unknown[] = [];
  /** Ops made while the socket opens. */
  const waiting: Record<string, unknown>[] = [];
  const emit = (events: unknown[]) => {
    if (events.length === 0) return;
    if (deliver) deliver(events);
    else early.push(...events);
  };
  const refuse = (ops: readonly Record<string, unknown>[], message: string) =>
    emit(ops.filter((o) => ASKS.has(String(o["op"]))).map((o) => ({ type: "error", op: String(o["op"]), code: "offline", message })));

  const socket = new Socket(url);
  socket.onopen = () => {
    opened = true;
    state = "open";
    for (const op of waiting.splice(0)) socket.send(JSON.stringify(op));
    if (closing) finish();
  };
  socket.onmessage = (e) => {
    if (typeof e.data !== "string") return;
    let event: unknown;
    try {
      event = JSON.parse(e.data);
    } catch {
      return;
    }
    emit([event]);
  };
  const lost = (why: string) => {
    if (state === "closed") return;
    state = "closed";
    if (closing) return;
    if (opened) emit([{ type: "left", reason: why }]);
    else refuse(waiting.splice(0), `cannot reach the relay at ${url}; start one with \`og relay\``);
  };
  socket.onclose = (e) => lost(e.wasClean ? "the relay closed the connection" : "the relay connection failed");
  socket.onerror = () => lost("the relay connection failed");
  const finish = () => {
    state = "closed";
    try {
      socket.close(1000);
    } catch {
      /* already closing */
    }
  };

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
      if (state === "closed" || closing) return refuse(ops, `the relay connection at ${url} is closed`);
      if (state === "connecting") return void waiting.push(...ops);
      for (const op of ops) socket.send(JSON.stringify(op));
    },
    close() {
      if (closing || state === "closed") return;
      closing = true;
      // While connecting: the waiting ops go when it opens, and then it closes.
      if (state === "open") finish();
    },
  };
}
