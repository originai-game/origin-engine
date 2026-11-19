// One page of tests/web-shared.test.ts, in its own process (the Worker owns
// the global scope): the real engine Worker (src/web/worker.ts) and shell
// (src/web/shell.ts) on the native host, the shared-walk demo as the game,
// and the page's session provider: tokenEndpoint(<endpoint>) from
// @origingame/engine/provider, or none. The Worker opens the world server's
// WebSocket itself (bun's, as a browser's).
//
// Bun has no WebTransport, so a grant's `webtransport` falls back to the
// WebSocket unless <transport> installs a stand-in: `wt=<ws URL>` is a
// WebTransport carried over the world server's WebSocket (snapshots arrive
// as datagrams, every other message as stream frames cut at arbitrary
// points; it counts any datagram the Worker sends that is not a proposal,
// and any proposal it frames), `wt-fail` one that never opens.
//
//   bun tests/support/shared-page.ts <game bundle> <project root, cooked> <token endpoint URL | none> <ticks> [<transport>]
import { readFileSync } from "node:fs";
import { loadNativeHost } from "../../src/node/index.ts";
import { tokenEndpoint } from "../../src/provider.ts";
import type { GameToken } from "../../src/runtime.ts";
import { Shell } from "../../src/web/shell.ts";
import { startWorker } from "../../src/web/worker.ts";

const [bundle, root, endpoint, ticks, transport = ""] = process.argv.slice(2) as [string, string, string, string, string?];

/** What the WebTransport stand-in saw. */
const wt = { opened: 0, datagramsOut: 0, datagramsIn: 0, misrouted: 0 };
const PROPOSE = 2;
const SNAPSHOT = 3;
if (transport.startsWith("wt")) {
  const target = transport.slice("wt=".length);
  const failed = () => {
    const p = Promise.reject(new Error("no answer"));
    p.catch(() => {});
    return p;
  };
  (globalThis as Record<string, unknown>)["WebTransport"] = class {
    readonly ready: Promise<void>;
    readonly closed: Promise<{ reason?: string }>;
    readonly datagrams: { readable: ReadableStream<Uint8Array>; writable: WritableStream<Uint8Array>; maxDatagramSize: number };
    #stream: ReadableStreamDefaultController<Uint8Array> | undefined;
    #dgrams: ReadableStreamDefaultController<Uint8Array> | undefined;
    #ws: WebSocket | undefined;
    constructor(url: string, options: { serverCertificateHashes?: { algorithm: string; value: Uint8Array }[] }) {
      if (!url.startsWith("https://") || options.serverCertificateHashes?.some((h) => h.algorithm !== "sha-256" || h.value.length !== 32)) {
        throw new TypeError(`a WebTransport to ${url} with ${JSON.stringify(options)}`);
      }
      const readable = new ReadableStream<Uint8Array>({ start: (c) => void (this.#dgrams = c) });
      const writable = new WritableStream<Uint8Array>({
        write: (b) => {
          wt.datagramsOut++;
          if (b[0] !== PROPOSE) wt.misrouted++;
          this.#ws?.send(b as Uint8Array<ArrayBuffer>);
        },
      });
      this.datagrams = { readable, writable, maxDatagramSize: 1200 };
      if (transport === "wt-fail") {
        this.ready = failed();
        this.closed = failed();
        return;
      }
      const ws = new WebSocket(target);
      ws.binaryType = "arraybuffer";
      this.#ws = ws;
      this.ready = new Promise((ok, no) => {
        ws.onopen = () => (wt.opened++, ok());
        ws.onerror = () => no(new Error("the stand-in's WebSocket failed"));
      });
      this.ready.catch(() => {});
      this.closed = new Promise((ok) => (ws.onclose = (e) => ok({ reason: e.reason })));
      let cut = 1;
      ws.onmessage = (e) => {
        const b = new Uint8Array(e.data as ArrayBuffer);
        if (b[0] === SNAPSHOT) {
          wt.datagramsIn++;
          this.#dgrams?.enqueue(b);
          return;
        }
        // A frame in two chunks, cut at a point that walks through the length and the body.
        const f = new Uint8Array(4 + b.length);
        new DataView(f.buffer).setUint32(0, b.length, true);
        f.set(b, 4);
        cut = (cut % f.length) + 1;
        this.#stream?.enqueue(f.slice(0, cut));
        if (cut < f.length) this.#stream?.enqueue(f.slice(cut));
      };
    }
    async createBidirectionalStream() {
      let buf = new Uint8Array(0);
      return {
        readable: new ReadableStream<Uint8Array>({ start: (c) => void (this.#stream = c) }),
        writable: new WritableStream<Uint8Array>({
          write: (chunk) => {
            const joined = new Uint8Array(buf.length + chunk.length);
            joined.set(buf);
            joined.set(chunk, buf.length);
            buf = joined;
            while (buf.length >= 4) {
              const len = new DataView(buf.buffer, buf.byteOffset, 4).getUint32(0, true);
              if (buf.length < 4 + len) break;
              const m = buf.slice(4, 4 + len);
              if (m[0] === PROPOSE) wt.misrouted++;
              this.#ws?.send(m as Uint8Array<ArrayBuffer>);
              buf = buf.subarray(4 + len);
            }
          },
        }),
      };
    }
    close(): void {
      this.#ws?.close();
    }
  };
}

function anything(): any {
  const f = () => {};
  return new Proxy(f, {
    get: (_, key) => (key === Symbol.toPrimitive ? () => "" : key === "then" ? undefined : anything()),
    set: () => true,
    apply: () => anything(),
  });
}
function canvas(): HTMLCanvasElement {
  const target = new EventTarget();
  return Object.assign(target, {
    clientWidth: 320,
    clientHeight: 180,
    width: 0,
    height: 0,
    ownerDocument: anything(),
    transferControlToOffscreen: () => ({ offscreen: true }),
    getBoundingClientRect: () => ({ left: 0, top: 0, width: 320, height: 180 }),
  }) as unknown as HTMLCanvasElement;
}
type Scope = { onmessage: ((e: { data: unknown }) => void) | null; postMessage(m: unknown): void };
const scope = globalThis as unknown as Scope;
class Port extends EventTarget {
  constructor() {
    super();
    scope.postMessage = (m: unknown) => setTimeout(() => this.dispatchEvent(new MessageEvent("message", { data: m })), 0);
  }
  postMessage(m: unknown): void {
    setTimeout(() => scope.onmessage?.({ data: m }), 0);
  }
  terminate(): void {}
}

(globalThis as Record<string, unknown>)["HTMLInputElement"] ??= class {};
// The Worker fetches its bundle's manifest by path; the page's origin is this disk.
const webFetch = globalThis.fetch;
globalThis.fetch = (async (url: string | URL | Request, init?: RequestInit) =>
  typeof url === "string" && url.startsWith("/") ? new Response(new Uint8Array(readFileSync(url))) : webFetch(url, init)) as typeof fetch;
loadNativeHost();
const game = ((await import(bundle)) as { default: GameToken }).default;
startWorker(game);
const shell = await Shell.start({ canvas: canvas(), worker: new Port() as unknown as Worker, glue: new URL("../fixtures/web-glue/native.ts", import.meta.url).href, wasm: "unused.wasm", autoplay: false, root });
shell.setSessions(endpoint === "none" ? null : tokenEndpoint(endpoint));

// Real time, as a page's frames keep it: the server's 20 Hz and this page's proposals meet.
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const t0 = performance.now();
let tick = 0;
let maxInSight = 0;
while (tick < Number(ticks)) {
  await shell.call("advance", 1);
  tick = ((await shell.call("frame", 0)) as { tick: number }).tick;
  maxInSight = Math.max(maxInSight, ((await shell.call("shared")) as { inSight: number }).inSight);
  const ahead = (tick * 1000) / 60 - (performance.now() - t0);
  await sleep(Math.max(1, ahead));
}
const shared = await shell.call("shared");
process.stdout.write(JSON.stringify({ tick, maxInSight, shared, wt }) + "\n");
process.exit(0);
