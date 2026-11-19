// og.shared's sessions in the engine Worker (docs/design.md §2.12). The page's
// session provider (provider.ts SessionProvider) names the world server and a
// token for each connection; the Worker opens the WebSocket itself, so the
// snapshots never cross the page.
//
// - The engine's ops (og_shared_take): `connect` asks the page (a `shared`
//   message) and the page's `shared-session` answers it; `send` goes out on
//   the open socket as one binary frame; `close` closes it.
// - What reaches the engine (og_shared_deliver), in order: `host` (the
//   provider's name, `none` without one), `open` with the token, every
//   binary frame as a `message`, `closed` (a lost connection, or a provider
//   failure it may try again) and `refused` (a provider failure that ends
//   the session). The engine reconnects; each `connect` opens a new socket.
// - Ops are taken after every frame and after each delivery, so the hello
//   leaves on the open that made it.

import { boundHost, type Engine } from "../../runtime.ts";
import { retryable } from "../../provider.ts";
import type { WorkerScope } from "./messages.ts";

declare const self: WorkerScope;

interface Socket {
  binaryType: string;
  readonly readyState: number;
  onopen: (() => void) | null;
  onmessage: ((e: { data: unknown }) => void) | null;
  onclose: ((e: { code: number; reason: string }) => void) | null;
  onerror: (() => void) | null;
  send(data: Uint8Array): void;
  close(): void;
}

/** The page's answer to one `connect`; code `none`: the page has no session provider. */
export type SessionGrant = { ok: true; url: string; token?: string } | { ok: false; code: string; message: string };

let socket: Socket | null = null;
/** The `connect` the page's answer is for: an older one's answer is dropped. */
let asked = 0;

function deliver(engine: Engine, event: Record<string, unknown>): void {
  boundHost()?.abi.og_shared_deliver(engine.id, JSON.stringify(event));
  pumpShared(engine);
}

function drop(): void {
  if (!socket) return;
  socket.onopen = socket.onmessage = socket.onclose = socket.onerror = null;
  socket.close();
  socket = null;
}

function toBase64(bytes: Uint8Array): string {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}

function fromBase64(text: string): Uint8Array {
  const s = atob(text);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

/** The engine's session ops: a connect to the page, sends and closes to the socket. */
export function pumpShared(engine: Engine): void {
  const host = boundHost();
  if (!host) return;
  const ops = JSON.parse(host.abi.og_shared_take(engine.id)) as { op: string; world?: string; reconnect?: boolean; bytes?: string }[];
  for (const op of ops) {
    if (op.op === "connect") {
      drop();
      asked++;
      self.postMessage({ type: "shared", request: asked, world: op.world ?? "main", reconnect: op.reconnect === true });
    } else if (op.op === "send" && op.bytes !== undefined) {
      if (socket?.readyState === 1) socket.send(fromBase64(op.bytes));
    } else if (op.op === "close") {
      asked++;
      drop();
    }
  }
}

/** The page's provider: its name (`none` without one). */
export function sharedHost(engine: Engine, name: string): void {
  deliver(engine, { type: "host", host: name });
}

/** The page answered `connect` number `request`: open the socket it names, or say why not. */
export function sharedSession(engine: Engine, request: number, grant: SessionGrant): void {
  if (request !== asked) return;
  if (!grant.ok && grant.code === "none") {
    // As natively: og.SharedLeft says `unavailable`.
    deliver(engine, { type: "host", host: "none" });
    return;
  }
  if (!grant.ok) {
    deliver(engine, retryable(grant.code) ? { type: "closed", reason: `${grant.code}: ${grant.message}` } : { type: "refused", code: grant.code, message: grant.message });
    return;
  }
  const Ctor = (globalThis as unknown as { WebSocket?: new (url: string) => Socket }).WebSocket;
  if (!Ctor) {
    deliver(engine, { type: "refused", code: "unavailable", message: "this browser has no WebSocket" });
    return;
  }
  let ws: Socket;
  try {
    ws = new Ctor(grant.url);
  } catch (e) {
    deliver(engine, { type: "refused", code: "invalid", message: `the provider's URL ${grant.url}: ${e instanceof Error ? e.message : String(e)}` });
    return;
  }
  ws.binaryType = "arraybuffer";
  socket = ws;
  ws.onopen = () => deliver(engine, { type: "open", transport: "ws", token: grant.token ?? "" });
  ws.onmessage = (e) => {
    if (e.data instanceof ArrayBuffer) deliver(engine, { type: "message", bytes: toBase64(new Uint8Array(e.data)) });
  };
  ws.onclose = (e) => {
    if (socket !== ws) return;
    socket = null;
    deliver(engine, { type: "closed", reason: e.reason || `the connection closed (${e.code})` });
  };
  // A failed socket also closes, and onclose says so.
  ws.onerror = () => {};
}
