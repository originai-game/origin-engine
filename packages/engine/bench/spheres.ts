// The sphere field of tools/bench/spheres (20,000 spheres, each moved every
// tick by a TS system; the per-object half of tools/bench/threejs-compare.ts)
// on the native host: frame times split into the tick (step) and extraction
// plus og-render (render), from the probe's state timings.
//
//   bun bench/spheres.ts [--frames 300] [--count 20000] [--size 1280x720]
//
// Needs a release addon (OG_RELEASE=1 sh scripts/build.sh, or the release
// addon copied to native/ with native/build-profile "release"). Prints one
// JSON line.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost, native } from "../src/node/index.ts";
import { bundleGame } from "../src/node/build.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

type Stats = { p50: number; p95: number; mean: number };
const r3 = (v: number) => Math.round(v * 1000) / 1000;
function stats(xs: number[]): Stats {
  const s = [...xs].sort((a, b) => a - b);
  return { p50: r3(s[Math.floor(s.length / 2)]!), p95: r3(s[Math.min(s.length - 1, Math.floor(0.95 * s.length))]!), mean: r3(xs.reduce((a, c) => a + c, 0) / xs.length) };
}

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tools", "bench", "spheres");
const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const [W, H] = opt("--size", "1280x720").split("x").map(Number) as [number, number];
const FRAMES = Number(opt("--frames", "300"));
const COUNT = Number(opt("--count", "20000"));
const WARM = 30;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const profileFile = join(pkg, "native", "build-profile");
const build = existsSync(profileFile) ? readFileSync(profileFile, "utf8").trim() : "unknown";
if (build !== "release" && !argv.includes("--any-build")) throw new Error(`the native addon is a ${build} build; frame times need release`);
loadNativeHost();
// The field's size is a constant of the game; a copy of its source sets it.
const work = join(repo, "target", "tmp", "spheres-bench");
mkdirSync(join(work, "src"), { recursive: true });
const source = readFileSync(join(project, "src", "main.ts"), "utf8").replace("export const COUNT = 20_000;", `export const COUNT = ${COUNT};`);
writeFileSync(join(work, "src", "main.ts"), source);
const bundled = join(work, `spheres-${process.pid}.js`);
const b = await bundleGame(join(work, "src", "main.ts"), bundled, "native");
if (!b.ok) throw new Error(b.reason);
const game = (await import(bundled)).default as GameToken;
const engine = must(createEngine(game, { root: project }));
const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
const wait = () => {
  const r = JSON.parse(native().og_node_wait_gpu(engine.id)) as { ok: boolean; reason?: string };
  if (!r.ok) throw new Error(r.reason);
};
const times: number[] = [], cpu: number[] = [];
for (let i = -WARM; i < FRAMES; i++) {
  const t0 = performance.now();
  const f = engine.frame(1000 / 60);
  if (!f.ok) throw new Error(f.reason);
  const t1 = performance.now();
  wait();
  const t2 = performance.now();
  if (i >= 0) { times.push(t2 - t0); cpu.push(t1 - t0); }
}
console.log(JSON.stringify({ host: "native", build, tier: attached.value, size: `${W}x${H}`, count: COUNT, frames: FRAMES, ms: stats(times), cpuMs: stats(cpu) }));
engine.destroy();
