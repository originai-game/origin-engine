// One host per process: the hit-volumes fixture's own game (a walker spun
// with an og.HitVolume on its hand, thin dummies and a wall on its circle,
// levers for og.play.candidates) advanced --ticks. Prints JSON for
// play.test.ts: the world hash and the game's facts.
//
//   bun tests/support/play-hash.ts --host node|web --project <tests/fixtures/hit-volumes> [--ticks 180]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const which = opt("--host", "node");
const ticks = Number(opt("--ticks", "180"));
const project = opt("--project", "");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

let root = project;
if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = (await import(join(pkg, "dist", "web", "origin-engine.js"))) as Record<string, (...a: unknown[]) => unknown>;
  await loadWebHost(glue as never, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
  // The Worker's preload, by hand: the manifest and every blob under <root>/cooked/.
  root = "game";
  const manifest = readFileSync(join(project, "cooked", ".cook.ogt"), "utf8");
  glue["og_web_put_file"]!(`${root}/cooked/.cook.ogt`, new TextEncoder().encode(manifest));
  const listed = JSON.parse(glue["og_web_bundle_files"]!(manifest) as string) as Array<{ file: string }>;
  for (const o of listed) glue["og_web_put_file"]!(`${root}/cooked/${o.file}`, new Uint8Array(readFileSync(join(project, "cooked", o.file))));
}

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const fixture = (await import(resolve(project, "src", "main.ts"))) as { default: unknown; facts: (world: World) => Record<string, number> };
const engine = must(createEngine(fixture.default as GameToken, { root }));
const tick = must(engine.advance(ticks));
console.log(JSON.stringify({ host: which, tick, hash: engine.hash(), facts: fixture.facts(engine.world) }));
