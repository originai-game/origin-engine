// The web relay room provider (src/web/relay-rooms.ts): og-net's room
// protocol over a WebSocket, as the native client speaks it. Against a
// stand-in relay in this process (the protocol's ops and events, rooms by
// path, per-sender order) it is asserted that ops made while the socket
// opens go first and in order, a room joined by the URL's path reaches the
// engine before it sent any op, one peer's messages reach the other in order
// with its departure after the last of them (close() sends what is queued
// first), binary and non-JSON frames are ignored, a lost connection is
// mended (the peer is back in its room under its name, an unanswered ask is
// asked again, what was sent meanwhile is lost), a relay that stays away is
// a `left` once the window passed and a later ask tries it again, and an
// unreachable one answers the join `offline`. With an `og` (OG_BIN or
// native/), the same provider is checked against the real `og relay`,
// including its 64 KiB limit and a quick play room taking its peer back.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { relayRooms, type RelayRooms, type SocketFactory } from "../src/web/relay-rooms.ts";
import { ogBin } from "./support/og-bin.ts";

type Socket = { data: { path: string; room?: string; peer?: number; name?: string }; send(text: string | Uint8Array): void; close(code?: number): void };
declare const Bun: {
  serve(options: {
    port: number;
    hostname: string;
    fetch(request: Request, server: { upgrade(r: Request, o: { data: { path: string } }): boolean }): Response | undefined;
    websocket: { open(ws: Socket): void; message(ws: Socket, text: string | Uint8Array): void; close(ws: Socket): void };
  }): { port: number; stop(force: boolean): void };
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): { stdout: ReadableStream<Uint8Array>; kill(): void };
};

/** Just enough of `og relay`: rooms by path or join, create, send to one or all, leave, peer_left on close. */
function standInRelay() {
  const rooms = new Map<string, { next: number; peers: Map<number, Socket> }>();
  const ops: unknown[] = [];
  let created = 0;
  const event = (ws: Socket, e: unknown) => ws.send(JSON.stringify(e));
  const live = new Set<Socket>();
  const join = (ws: Socket, op: string, room: string, name = "") => {
    const r = rooms.get(room) ?? { next: 0, peers: new Map() };
    rooms.set(room, r);
    const peer = r.next++;
    for (const other of r.peers.values()) event(other, { type: "peer_joined", peer, name });
    r.peers.set(peer, ws);
    ws.data.room = room;
    ws.data.peer = peer;
    ws.data.name = name;
    event(ws, { type: "joined", room, peer, peers: [...r.peers].map(([p, s]) => ({ peer: p, name: s.data.name ?? "" })) });
    void op;
  };
  const leave = (ws: Socket) => {
    const r = ws.data.room === undefined ? undefined : rooms.get(ws.data.room);
    if (!r) return;
    r.peers.delete(ws.data.peer!);
    for (const other of r.peers.values()) event(other, { type: "peer_left", peer: ws.data.peer });
    delete ws.data.room;
  };
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    fetch(req, s) {
      return s.upgrade(req, { data: { path: new URL(req.url).pathname } }) ? undefined : new Response("upgrade", { status: 400 });
    },
    websocket: {
      open(ws) {
        live.add(ws);
        const room = ws.data.path.replace(/^\/+|\/+$/g, "");
        if (room === "binary") {
          ws.send(new Uint8Array([1, 2, 3]));
          ws.send("not json");
          ws.send(JSON.stringify({ type: "left", reason: "after junk" }));
        } else if (room === "kick") ws.close(1000);
        else if (room) join(ws, "join", room);
      },
      message(ws, text) {
        if (typeof text !== "string") return;
        const op = JSON.parse(text) as Record<string, unknown>;
        ops.push(op);
        if (op["op"] === "create") join(ws, "create", `room-${++created}`);
        else if (op["op"] === "join") join(ws, "join", String(op["room"]), String(op["name"] ?? ""));
        else if (op["op"] === "leave") {
          leave(ws);
          event(ws, { type: "left", reason: "left" });
        } else if (op["op"] === "send") {
          const r = rooms.get(ws.data.room ?? "");
          if (!r) return event(ws, { type: "error", op: "send", code: "invalid", message: "not in a room: join one first" });
          const { op: _, to, ...rest } = op;
          for (const [p, other] of r.peers) if (p !== ws.data.peer && (to === undefined || to === p)) event(other, { type: "message", from: ws.data.peer, ...rest });
        }
      },
      close(ws) {
        live.delete(ws);
        leave(ws);
      },
    },
  });
  /** Ends every connection whose path is not `keep`, as a relay that restarts does. */
  const cut = (keep: string) => {
    for (const ws of [...live]) if (ws.data.path !== keep) ws.close(1012);
  };
  return { url: `ws://127.0.0.1:${server.port}`, ops, cut, stop: () => server.stop(true) };
}

/** The events a provider delivered, and a wait for the n-th. */
function listen(rooms: RelayRooms) {
  const got: Array<Record<string, unknown>> = [];
  const deliver = (events: unknown[]) => got.push(...(events as Array<Record<string, unknown>>));
  return {
    got,
    deliver,
    async until(n: number, ms = 3000) {
      const end = Date.now() + ms;
      while (got.length < n && Date.now() < end) await new Promise((r) => setTimeout(r, 2));
      if (got.length < n) throw new Error(`waited for ${n} events, got ${JSON.stringify(got)}`);
      return got;
    },
  };
}

test("ops wait for the socket in order; a path's room arrives before any op; messages keep their order and the departure follows them", async () => {
  const relay = standInRelay();
  try {
    // a joins by path and hears it through attach, before it sent anything.
    const a = relayRooms(`${relay.url}/arena`);
    expect(a.host).toBe("relay");
    expect(a.state).toBe("connecting");
    const ha = listen(a);
    a.attach!(ha.deliver);
    expect((await ha.until(1))[0]).toEqual({ type: "joined", room: "arena", peer: 0, peers: [{ peer: 0, name: "" }] });
    expect(a.state).toBe("open");
    // b sends a join and messages at once, while it connects: they go in order.
    const b = relayRooms(`${relay.url}/`);
    const hb = listen(b);
    b.handle([{ op: "join", room: "arena", name: "b" }, ...[0, 1, 2].map((i) => ({ op: "send", data: { early: i } }))], hb.deliver);
    expect((await hb.until(1))[0]).toMatchObject({ type: "joined", room: "arena", peer: 1 });
    expect(relay.ops.slice(0, 4)).toEqual([{ op: "join", room: "arena", name: "b" }, { op: "send", data: { early: 0 } }, { op: "send", data: { early: 1 } }, { op: "send", data: { early: 2 } }]);
    await ha.until(5);
    expect(ha.got.slice(1, 5)).toEqual([{ type: "peer_joined", peer: 1, name: "b" }, ...[0, 1, 2].map((i) => ({ type: "message", from: 1, data: { early: i } }))]);
    // 200 messages and bytes on the engine's channel, then close(): a hears them all, then b's departure.
    b.handle(Array.from({ length: 200 }, (_, i) => ({ op: "send", data: i })), hb.deliver);
    b.handle([{ op: "send", to: 0, channel: "og.lockstep", bytes: "AQID" }], hb.deliver);
    b.close();
    const all = await ha.until(1 + 1 + 3 + 200 + 1 + 1);
    const tail = all.slice(5);
    expect(tail.slice(0, 200).map((e) => e["data"])).toEqual(Array.from({ length: 200 }, (_, i) => i));
    expect(tail[200]).toEqual({ type: "message", from: 1, channel: "og.lockstep", bytes: "AQID" });
    expect(tail[201]).toEqual({ type: "peer_left", peer: 1 });
    // A closed provider answers a later join offline (and sends nothing).
    b.handle([{ op: "join", room: "arena" }, { op: "send", data: 1 }], hb.deliver);
    expect(hb.got.at(-1)).toMatchObject({ type: "error", op: "join", code: "offline" });
    expect(b.state).toBe("closed");
    a.close();
  } finally {
    relay.stop();
  }
});

test("binary and non-JSON frames are ignored; a relay that closes is a left; an unreachable one answers the waiting join offline", async () => {
  const relay = standInRelay();
  try {
    const junk = relayRooms(`${relay.url}/binary`);
    const hj = listen(junk);
    junk.attach!(hj.deliver);
    expect(await hj.until(1)).toEqual([{ type: "left", reason: "after junk" }]);
    junk.close();

    // With no window to try again in, a relay that closes is the room lost at once; a later ask tries it once more.
    const kicked = relayRooms(`${relay.url}/kick`, { reconnectMs: 0 });
    const hk = listen(kicked);
    kicked.attach!(hk.deliver);
    expect(await hk.until(1)).toEqual([{ type: "error", op: "join", code: "offline", message: expect.stringContaining("the relay closed the connection") }]);
    expect(kicked.state).toBe("closed");
    kicked.handle([{ op: "send", data: "lost" }, { op: "create" }], hk.deliver);
    expect(kicked.state).toBe("connecting");
    expect((await hk.until(2))[1]).toMatchObject({ type: "joined", room: "room-1", peer: 0 });
    kicked.close();
  } finally {
    relay.stop();
  }
  // Nothing listens there now.
  const gone = relayRooms(relay.url + "/");
  const hg = listen(gone);
  gone.handle([{ op: "join", room: "arena" }, { op: "send", data: 1 }, { op: "quick", mode: "duel" }], hg.deliver);
  const got = await hg.until(2);
  expect(got.map((e) => [e["op"], e["code"]])).toEqual([
    ["join", "offline"],
    ["quick", "offline"],
  ]);
  expect(String(got[0]!["message"])).toContain("cannot reach the relay");
  expect(() => relayRooms("http://127.0.0.1:1/")).toThrow("is not a relay URL");
});

test("a lost connection is mended: the peer is back in its room under its name, an unanswered ask is asked again; a relay that stays away is a left", async () => {
  const relay = standInRelay();
  try {
    const a = relayRooms(`${relay.url}/watch`);
    const ha = listen(a);
    a.handle([{ op: "leave" }, { op: "join", room: "arena", name: "ann" }], ha.deliver);
    await ha.until(3);
    const b = relayRooms(`${relay.url}/`, { reconnectMs: 1000 });
    const hb = listen(b);
    b.handle([{ op: "join", room: "arena", name: "bob" }], hb.deliver);
    expect((await hb.until(1))[0]).toMatchObject({ type: "joined", room: "arena", peer: 1 });
    await ha.until(4);
    const seen = relay.ops.length;

    relay.cut("/watch");
    // While it is away: a send is lost, an ask is refused (the engine is in its room), and nothing says `left`.
    await new Promise((r) => setTimeout(r, 20));
    expect(b.state).toBe("connecting");
    b.handle([{ op: "send", data: "lost" }, { op: "create" }], hb.deliver);
    expect(hb.got[1]).toMatchObject({ type: "error", op: "create", code: "offline" });
    expect((await hb.until(3))[2]).toEqual({ type: "joined", room: "arena", peer: 2, peers: [{ peer: 0, name: "ann" }, { peer: 2, name: "bob" }] });
    expect(b.state).toBe("open");
    expect(relay.ops.slice(seen)).toEqual([{ op: "join", room: "arena", name: "bob", was: 1 }]);
    expect((await ha.until(6)).slice(4)).toEqual([{ type: "peer_left", peer: 1 }, { type: "peer_joined", peer: 2, name: "bob" }]);
    b.handle([{ op: "send", data: "back" }], hb.deliver);
    expect((await ha.until(7))[6]).toEqual({ type: "message", from: 2, data: "back" });

    // A leave while it is away is a `left` at once, and the ask after it is the one the new connection makes.
    relay.cut("/watch");
    await new Promise((r) => setTimeout(r, 20));
    b.handle([{ op: "leave" }, { op: "join", room: "lobby", name: "bob" }], hb.deliver);
    expect(hb.got[3]).toEqual({ type: "left", reason: "left" });
    expect((await hb.until(5))[4]).toMatchObject({ type: "joined", room: "lobby", peer: 0 });
    expect((await ha.until(8))[7]).toEqual({ type: "peer_left", peer: 2 });
    a.close();

    // The relay stays away: the room is lost once the window passed, and not before the provider tried again.
    relay.stop();
    const at = Date.now();
    expect((await hb.until(6))[5]).toEqual({ type: "left", reason: "the relay closed the connection" });
    expect(Date.now() - at).toBeGreaterThanOrEqual(250);
    expect(b.state).toBe("closed");
    b.handle([{ op: "quick", mode: "duel" }], hb.deliver);
    expect((await hb.until(7))[6]).toMatchObject({ type: "error", op: "quick", code: "offline" });
  } finally {
    relay.stop();
  }
});

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const og = (() => {
  try {
    return ogBin();
  } catch {
    return null;
  }
})();

test.skipIf(!og)("against the real og relay: two pages meet by create and join, keep order, and a game message over 64 KiB is too_large", async () => {
  const p = Bun.spawn([og!, "relay", "--port", "0"], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  try {
    const reader = p.stdout.getReader();
    let text = "";
    while (!/:(\d+)\//.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og relay exited: ${text}`);
      text += new TextDecoder().decode(value);
    }
    const url = `ws://127.0.0.1:${/:(\d+)\//.exec(text)![1]}/`;
    const a = relayRooms(url);
    const ha = listen(a);
    a.handle([{ op: "create", name: "a", capacity: 2 }], ha.deliver);
    const joined = (await ha.until(1))[0]!;
    expect(joined).toMatchObject({ type: "joined", peer: 0, peers: [{ peer: 0, name: "a" }] });
    const b = relayRooms(url);
    const hb = listen(b);
    b.handle([{ op: "join", room: String(joined["room"]), name: "b" }, ...Array.from({ length: 50 }, (_, i) => ({ op: "send", data: i }))], hb.deliver);
    expect((await hb.until(1))[0]).toMatchObject({ type: "joined", peer: 1 });
    b.handle([{ op: "send", data: "x".repeat(64 * 1024) }], hb.deliver);
    expect((await hb.until(2))[1]).toMatchObject({ type: "error", op: "send", code: "too_large" });
    b.close();
    const got = await ha.until(1 + 1 + 50 + 1);
    expect(got[1]).toEqual({ type: "peer_joined", peer: 1, name: "b" });
    expect(got.slice(2, 52).map((e) => e["data"])).toEqual(Array.from({ length: 50 }, (_, i) => i));
    expect(got[52]).toEqual({ type: "peer_left", peer: 1 });
    a.close();
  } finally {
    p.kill();
  }
});

test.skipIf(!og)("against the real og relay: a quick play room takes its matched page back after a lost connection, and nobody without its key", async () => {
  const p = Bun.spawn([og!, "relay", "--port", "0"], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  try {
    const reader = p.stdout.getReader();
    let text = "";
    while (!/:(\d+)\//.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og relay exited: ${text}`);
      text += new TextDecoder().decode(value);
    }
    const url = `ws://127.0.0.1:${/:(\d+)\//.exec(text)![1]}/`;
    const a = relayRooms(url);
    const ha = listen(a);
    a.handle([{ op: "quick", mode: "duel", players: 2, name: "ann" }], ha.deliver);
    // b's sockets are kept, so the test can end one under it.
    const sockets: Array<{ close(code?: number): void }> = [];
    const Kept = class extends (globalThis as unknown as { WebSocket: SocketFactory }).WebSocket {
      constructor(u: string) {
        super(u);
        sockets.push(this);
      }
    } as SocketFactory;
    const b = relayRooms(url, { WebSocket: Kept });
    const hb = listen(b);
    b.handle([{ op: "quick", mode: "duel", players: 2, name: "bob" }], hb.deliver);
    const first = (await hb.until(1))[0]!;
    expect(first).toMatchObject({ type: "joined", peer: 1 });
    expect(String(first["room"])).toMatch(/^quick-duel-\d+$/);
    await ha.until(1);
    sockets[0]!.close();
    expect((await hb.until(2))[1]).toMatchObject({ type: "joined", room: first["room"], peer: 2, peers: [{ peer: 0, name: "ann" }, { peer: 2, name: "bob" }] });
    expect((await ha.until(3)).slice(1)).toEqual([{ type: "peer_left", peer: 1 }, { type: "peer_joined", peer: 2, name: "bob", was: 1 }]);
    b.close();
    await ha.until(4);
    // The place is free again, and a stranger who knows the room's id and a player's name still gets none.
    const c = relayRooms(url);
    const hc = listen(c);
    c.handle([{ op: "join", room: String(first["room"]), name: "bob" }], hc.deliver);
    expect((await hc.until(1))[0]).toMatchObject({ type: "error", op: "join", code: "full" });
    c.close();
    a.close();
  } finally {
    p.kill();
  }
});
