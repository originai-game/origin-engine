// og.shared's sessions in the engine Worker (docs/design.md §2.12). The page's
// session provider (provider.ts SessionProvider) names the world server and a
// token for each connection; the Worker opens the connection itself, so the
// snapshots never cross the page.
//
// - The connection is a WebSocket, or WebTransport when the grant names it
//   and the browser has it: proposals leave as datagrams and snapshots
//   arrive as datagrams; every other message crosses one bidirectional
//   stream the Worker opens, each as its length (u32 LE) and its bytes
//   (og-shared's wt.rs). A WebTransport that fails to open falls back to
//   the grant's WebSocket.
//
// - The engine's ops (og_shared_take): `connect` asks the page (a `shared`
//   message) and the page's `shared-session` answers it; `send` goes out on
//   the open socket as one binary frame; `close` closes it.
// - What reaches the engine (og_shared_deliver), in order: `host` (the
//   provider's name, `none` without one), `open` with the token and the
//   transport (`ws`, `wt`), every message as a `message`, `closed` (a lost connection, or a provider
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

/** A browser's WebTransport, written out: the Worker's type check has no DOM. */
interface StreamReader {
  read(): Promise<{ done: boolean; value?: Uint8Array }>;
}
interface StreamWriter {
  write(chunk: Uint8Array): Promise<void>;
}
interface Duplex {
  readonly readable: { getReader(): StreamReader };
  readonly writable: { getWriter(): StreamWriter };
}
interface Transport {
  readonly ready: Promise<void>;
  readonly closed: Promise<{ reason?: string }>;
  readonly datagrams: Duplex & { readonly maxDatagramSize?: number };
  createBidirectionalStream(): Promise<Duplex>;
  close(): void;
}
type TransportCtor = new (
  url: string,
  options: { serverCertificateHashes?: { algorithm: string; value: Uint8Array }[] },
) => Transport;

/** The page's answer to one `connect`; code `none`: the page has no session provider. */
export type SessionGrant =
  | { ok: true; url: string; token?: string; webtransport?: { url: string; certHashes?: readonly string[] } }
  | { ok: false; code: string; message: string };
type Granted = Extract<SessionGrant, { ok: true }>;

/** og-shared's C_PROPOSE: the one client message that crosses WebTransport as a datagram. */
const PROPOSE = 2;

/** The open connection: `send` once it is open, `close` drops it without a `closed`. */
interface Link {
  open: boolean;
  /** When a message last arrived (performance.now), and the silence watch's timer. */
  heard: number;
  watch?: ReturnType<typeof setInterval>;
  send(bytes: Uint8Array): void;
  close(): void;
}

/**
 * An open connection that hears nothing for SILENT_MS is lost (og-shared client.rs SILENT): a
 * world server sends a snapshot every tick, and a WebTransport client whose NAT rebinds is never
 * told. Measured on the Worker's clock, in real time, never in engine ticks. A watch that itself
 * ran late (the Worker busy with a long frame, a throttled background tab) restarts the count, so
 * a stall on this side is never taken for the server's silence.
 */
const SILENT_MS = 2000;
const WATCH_MS = 250;

function watch(engine: Engine, l: Link): void {
  let beat = performance.now();
  l.heard = beat;
  l.watch = setInterval(() => {
    const now = performance.now();
    if (now - beat > 4 * WATCH_MS) l.heard = now;
    beat = now;
    if (link !== l || now - l.heard <= SILENT_MS) return;
    link = null;
    l.close();
    deliver(engine, { type: "closed", reason: `the server was silent for ${SILENT_MS / 1000} s` });
  }, WATCH_MS);
}

let link: Link | null = null;
/** The `connect` the page's answer is for: an older one's answer is dropped. */
let asked = 0;

function deliver(engine: Engine, event: Record<string, unknown>): void {
  boundHost()?.abi.og_shared_deliver(engine.id, JSON.stringify(event));
  pumpShared(engine);
}

function drop(): void {
  const l = link;
  link = null;
  l?.close();
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
      if (link?.open) link.send(fromBase64(op.bytes));
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
  const Transport = (globalThis as unknown as { WebTransport?: TransportCtor }).WebTransport;
  if (grant.webtransport && Transport) openTransport(engine, grant, Transport);
  else openSocket(engine, grant);
}

/** WebTransport to the grant's `webtransport`; the grant's WebSocket when it fails to open. */
function openTransport(engine: Engine, grant: Granted, Ctor: TransportCtor): void {
  const wt = grant.webtransport!;
  let t: Transport;
  try {
    const hashes = (wt.certHashes ?? []).map((h) => ({ algorithm: "sha-256", value: fromHex(h) }));
    t = new Ctor(wt.url, hashes.length > 0 ? { serverCertificateHashes: hashes } : {});
  } catch {
    openSocket(engine, grant);
    return;
  }
  const l: Link = {
    open: false,
    heard: 0,
    send: () => {},
    close: () => {
      clearInterval(l.watch);
      t.close();
    },
  };
  link = l;
  const lost = (reason: string) => {
    if (link !== l) return;
    link = null;
    clearInterval(l.watch);
    if (l.open) deliver(engine, { type: "closed", reason });
    else openSocket(engine, grant);
  };
  const message = (bytes: Uint8Array) => {
    if (link !== l) return;
    l.heard = performance.now();
    deliver(engine, { type: "message", bytes: toBase64(bytes) });
  };
  void t.closed.then(
    (c) => lost(c.reason || "the WebTransport session closed"),
    (e: unknown) => lost(`the WebTransport session failed: ${e instanceof Error ? e.message : String(e)}`),
  );
  void (async () => {
    await t.ready;
    const stream = await t.createBidirectionalStream();
    if (link !== l) return;
    const out = stream.writable.getWriter();
    const datagrams = t.datagrams.writable.getWriter();
    const max = t.datagrams.maxDatagramSize ?? 1200;
    l.send = (b) => {
      const w = b[0] === PROPOSE && b.length <= max ? datagrams.write(b) : out.write(frame(b));
      w.catch(() => {});
    };
    l.open = true;
    watch(engine, l);
    deliver(engine, { type: "open", transport: "wt", token: grant.token ?? "" });
    void readFrames(stream.readable.getReader(), message).catch(() => {});
    const dg = t.datagrams.readable.getReader();
    for (;;) {
      const { done, value } = await dg.read();
      if (done || link !== l) return;
      if (value) message(value);
    }
  })().catch((e: unknown) => lost(`WebTransport: ${e instanceof Error ? e.message : String(e)}`));
}

/** One message as a stream frame: its length (u32 LE) and its bytes. */
function frame(bytes: Uint8Array): Uint8Array {
  const f = new Uint8Array(4 + bytes.length);
  new DataView(f.buffer).setUint32(0, bytes.length, true);
  f.set(bytes, 4);
  return f;
}

/** Each frame of a stream to `each`, until it ends. */
async function readFrames(reader: StreamReader, each: (bytes: Uint8Array) => void): Promise<void> {
  let buf = new Uint8Array(0);
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return;
    if (!value) continue;
    const joined = new Uint8Array(buf.length + value.length);
    joined.set(buf);
    joined.set(value, buf.length);
    buf = joined;
    while (buf.length >= 4) {
      const len = new DataView(buf.buffer, buf.byteOffset, 4).getUint32(0, true);
      if (buf.length < 4 + len) break;
      each(buf.slice(4, 4 + len));
      buf = buf.subarray(4 + len);
    }
  }
}

function fromHex(hex: string): Uint8Array {
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(2 * i, 2 * i + 2), 16);
  return out;
}

/** A WebSocket to the grant's URL. */
function openSocket(engine: Engine, grant: Granted): void {
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
  const l: Link = {
    open: false,
    heard: 0,
    send: (b) => ws.send(b),
    close: () => {
      clearInterval(l.watch);
      ws.onopen = ws.onmessage = ws.onclose = ws.onerror = null;
      ws.close();
    },
  };
  link = l;
  ws.onopen = () => {
    l.open = true;
    watch(engine, l);
    deliver(engine, { type: "open", transport: "ws", token: grant.token ?? "" });
  };
  ws.onmessage = (e) => {
    if (!(e.data instanceof ArrayBuffer)) return;
    l.heard = performance.now();
    deliver(engine, { type: "message", bytes: toBase64(new Uint8Array(e.data)) });
  };
  ws.onclose = (e) => {
    if (link !== l) return;
    link = null;
    clearInterval(l.watch);
    deliver(engine, { type: "closed", reason: e.reason || `the connection closed (${e.code})` });
  };
  // A failed socket also closes, and onclose says so.
  ws.onerror = () => {};
}
