// One page of tests/web-shared.test.ts, in its own process (the Worker owns
// the global scope): the real engine Worker (src/web/worker.ts) and shell
// (src/web/shell.ts) on the native host, the shared-walk demo as the game,
// and the page's session provider: tokenEndpoint(<endpoint>) from
// @origingame/engine/provider, or none. The Worker opens the world server's
// WebSocket itself (bun's, as a browser's).
//
//   bun tests/support/shared-page.ts <game bundle> <project root, cooked> <token endpoint URL | none> <ticks>
import { readFileSync } from "node:fs";
import { loadNativeHost } from "../../src/node/index.ts";
import { tokenEndpoint } from "../../src/provider.ts";
import type { GameToken } from "../../src/runtime.ts";
import { Shell } from "../../src/web/shell.ts";
import { startWorker } from "../../src/web/worker.ts";

const [bundle, root, endpoint, ticks] = process.argv.slice(2) as [string, string, string, string];

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
process.stdout.write(JSON.stringify({ tick, maxInSight, shared }) + "\n");
process.exit(0);
