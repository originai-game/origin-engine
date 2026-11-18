// One host per process (the runtime binds one): a world that opens a bundle
// per parcel with og.load.bundle and walks an anchor from cell to cell, on
// the host named by --host. On web (the wasm module, run here outside a
// browser) every cooked file reaches the engine as it does in a page: the
// Worker's boot preloads the game's bundle (boot.ts preloadBundle,
// transcodeBundle) and its fetcher (installFetcher) fetches the rest,
// transcoding the Basis textures of a streaming group's pack on the way.
// Prints JSON for runtime-bundles.test.ts: at each stop, the cell's group
// (its state and reason) and the budget's `used`. With --sign it walks
// nowhere: it loads a-v1's sign, a model in no group of its bundle, and
// prints whether it loaded and whether its texture arrived (og.loading.idle).
//
//   bun tests/support/runtime-bundles.ts --host node|web --dir <staged> [--sign]
//
// <staged>/game is the game's project; its parcels are projects of their
// own under <staged>/game/parcels/<id> (runtime-bundles.test.ts stages and
// cooks them).

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import type { Asset, World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => {
  const i = argv.indexOf(name);
  if (i < 0) throw new Error(`${name} is required`);
  return argv[i + 1]!;
};
const which = opt("--host");
const staged = opt("--dir");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

/** The parcels, opened in this order (each one's group is named after it), and where their cells are. */
const PARCELS = [
  ["a-v1", 100],
  ["a-v2", 300],
] as const;
/** The stops: a cell (its group's name) and where it is; the game's own `home` cell after the parcels, and nowhere last. */
const STOPS: ReadonlyArray<readonly [string | null, number]> = [...PARCELS, ["home", -100], [null, 1000]];

let root = join(staged, "game");
if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = (await import(join(pkg, "dist", "web", "origin-engine.js"))) as never;
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
  // The page: the staged directory served as is, the game at /game. A
  // request's `integrity` is checked as a browser does: bytes whose hash
  // differs are refused with a TypeError.
  root = "game";
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    const path = decodeURIComponent(String(input));
    let bytes: Uint8Array<ArrayBuffer>;
    try {
      bytes = new Uint8Array(readFileSync(join(staged, path)));
    } catch {
      return new Response(null, { status: 404 });
    }
    if (init?.integrity && init.integrity !== `sha384-${btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.digest("SHA-384", bytes))))}`) throw new TypeError("Failed to fetch");
    return new Response(bytes);
  }) as typeof fetch;
  const boot = await import("../../src/web/boot.ts");
  const fetched = await boot.preloadBundle(glue, root, () => {});
  if (fetched.failed > 0) throw new Error(`the boot preload failed ${fetched.failed} file(s)`);
  boot.installFetcher(glue, undefined, () => null, fetched.integrity);
  await boot.transcodeBundle(glue, null, fetched, undefined, () => {});
}

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

const game = og.game({ tick: 60, seed: 1, start: (world: World) => void must(world.instantiate(must(og.load.scene(world, "scenes/world.ogt")))) }) as unknown as GameToken;
const engine = must(createEngine(game, { root }));
const world = engine.world;

/** Frames (a 60 Hz display) until `done`, the fetches the web host waits on answered between them; then two more. False when 600 frames did not do. */
async function until(what: string, done: () => boolean): Promise<boolean> {
  for (let i = 0; i < 600; i++) {
    const frame = engine.frame(1000 / 60);
    if (!frame.ok) throw new Error(`${what}: ${frame.reason}`);
    await new Promise((ok) => setTimeout(ok, 1));
    if (done()) {
      for (let k = 0; k < 2; k++) {
        engine.frame(1000 / 60);
        await new Promise((ok) => setTimeout(ok, 1));
      }
      return true;
    }
  }
  return false;
}

async function loaded(asset: Asset, what: string): Promise<void> {
  if (!(await until(`${what} opens`, () => og.load.status(world, asset) !== "pending"))) throw new Error(`${what} does not open`);
  if (og.load.status(world, asset) !== "ready") throw new Error(`${what}: ${og.load.error(world, asset)}`);
}

for (const [id] of PARCELS) await loaded(must(og.load.bundle(world, `parcels/${id}/cooked`)), `parcels/${id}`);
/** On the web a runtime bundle's files are fetched on demand: the first load starts the fetch and says so, a later one finds it. */
async function fetchedLoad(what: string, load: () => ReturnType<typeof og.load.scene>): Promise<Asset> {
  return must(await fetching(what, load));
}
async function fetching(what: string, load: () => ReturnType<typeof og.load.scene>): Promise<ReturnType<typeof og.load.scene>> {
  let r = load();
  if (!r.ok && /still being fetched/.test(r.reason)) await until(`${what} is fetched`, () => (r = load()).ok || !/still being fetched/.test(r.reason));
  return r;
}
if (argv.includes("--sign")) {
  const r = await fetching("a-v1/sign.glb", () => og.load.gltf(world, "a-v1/sign.glb"));
  const sign = r.ok ? { loaded: true, prepared: await until("a-v1/sign.glb's texture arrives", () => og.loading.idle(world)) } : { loaded: false, reason: r.reason };
  console.log(JSON.stringify({ host: which, sign }));
  process.exit(0);
}
for (const [id] of PARCELS) must(world.instantiate(await fetchedLoad(`${id}/place.ogt`, () => og.load.scene(world, `${id}/place.ogt`))));
// Each house is loaded (its proxy drawn) before its cell is near, so the
// group's load puts the full model in place or fails with the reason. (A
// model a resident group's cell loads later is upgraded after, and a
// failure there leaves its proxy without a word: stream::refresh.)
for (const id of [...PARCELS.map(([p]) => p), "home"]) await fetchedLoad(`${id}/house.glb`, () => og.load.gltf(world, `${id}/house.glb`));

const player = world.find("player")!;
const cellLoaded = (id: string) => must(world.get(world.find(`cell-${id}`)!, og.StreamCell)).loaded;
const group = (id: string) => {
  const s = must(og.stream.status(world, id));
  return { state: s.state, reason: s.reason, ready: s.ready };
};
const gone = (id: string) => !cellLoaded(id) && ["unloaded", "failed"].includes(group(id).state);
const out: Array<Record<string, unknown>> = [];
for (const [id, x] of STOPS) {
  must(world.set(player, og.Transform, { position: { x, y: 0, z: 0 } }));
  // The cell there loads with its group resident and its textures ready (or the group fails), and the others' are let go.
  const others = STOPS.flatMap(([c]) => (c === null || c === id ? [] : [c]));
  const here = () => id === null || group(id).state === "failed" || (cellLoaded(id) && group(id).state === "resident" && group(id).ready);
  const settled = await until(id === null ? "every cell unloads" : `cell-${id} loads`, () => here() && others.every(gone));
  out.push({ at: id ?? "away", ...(id === null ? {} : { group: group(id) }), used: must(og.stream.budget(world)).used, ...(settled ? {} : { settled }) });
}
console.log(JSON.stringify({ host: which, stops: out }));
