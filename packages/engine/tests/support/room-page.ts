// One Play page of tests/web-rooms.test.ts, in its own process (the Worker
// owns the global scope): the real engine Worker (src/web/worker.ts) and
// shell (src/web/shell.ts) on the native host, the arena archetype's bundle
// as the game, and a Player stand-in whose room.* is the relay the test
// started, spoken over a WebSocket in og-net's protocol (the shapes
// docs/platform-asks.md #16 gives the Player). The room comes from the Play
// link (`setRooms(rooms, link)`, what shell/page.ts does with `?room=`):
// the game never calls roomJoin.
//
//   bun tests/support/room-page.ts <game bundle> <project root, cooked> <relay port> <room> <bot> <ticks>
import { readFileSync } from "node:fs";
import { loadNativeHost } from "../../src/node/index.ts";
import type { GameToken } from "../../src/runtime.ts";
import { Shell } from "../../src/web/shell.ts";
import { startWorker } from "../../src/web/worker.ts";
import { playerRooms, type PlayerLink } from "../../shell/platform.ts";

const [bundle, root, port, room, bot, ticks] = process.argv.slice(2) as [string, string, string, string, string, string];

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

// The accessibility mirror's DOM class checks, with no DOM.
(globalThis as Record<string, unknown>)["HTMLInputElement"] ??= class {};
// The Worker fetches its bundle's manifest by path; the page's origin is this disk.
const webFetch = globalThis.fetch;
globalThis.fetch = (async (url: string | URL | Request, init?: RequestInit) =>
  typeof url === "string" && url.startsWith("/") ? new Response(new Uint8Array(readFileSync(url))) : webFetch(url, init)) as typeof fetch;
loadNativeHost();
const game = ((await import(bundle)) as { default: GameToken }).default;
startWorker(game);
const shell = await Shell.start({ canvas: canvas(), worker: new Port() as unknown as Worker, glue: new URL("../fixtures/web-glue/native.ts", import.meta.url).href, wasm: "unused.wasm", autoplay: false, root });

// The Player's rooms: each room.* request is the op of the same name on the relay's socket.
const socket = new WebSocket(`ws://127.0.0.1:${port}/`);
const open = new Promise<void>((r) => socket.addEventListener("open", () => r()));
let push: ((e: Record<string, unknown>) => void) | undefined;
socket.addEventListener("message", (m) => push?.(JSON.parse(String(m.data))));
const player: PlayerLink = {
  framed: true,
  hosted: true,
  capabilities: ["room.create", "room.join", "room.leave", "room.send"],
  async request(method, args) {
    await open;
    socket.send(JSON.stringify({ op: method.slice("room.".length), ...args }));
    return { ok: true, data: null } as never;
  },
  event() {},
};
const rooms = playerRooms(player)!;
push = (e) => rooms.push(e);
await shell.call("command", "bot", [bot]);
shell.setRooms(rooms, room);

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
let tick = 0;
let joined = false;
for (let i = 0; tick < Number(ticks); i++) {
  if (i > 200_000) throw new Error(`stuck at tick ${tick}`);
  const taken = (await shell.call("advance", 1)) as { ok: boolean; value?: number };
  tick = ((await shell.call("frame", 0)) as { tick: number }).tick;
  if (!joined) joined = ((await shell.call("net")) as { room: { state: string } }).room.state === "in";
  // A tick lockstep held waits for the other page's input; the lobby keeps a frame's pace.
  if (!taken.value) await sleep(1);
  else if (!((await shell.call("net")) as { status: { running: boolean } }).status.running) await sleep(16);
}
const net = (await shell.call("net")) as Record<string, unknown>;
process.stdout.write(JSON.stringify({ tick, joined, net }) + "\n");
socket.close();
process.exit(0);
