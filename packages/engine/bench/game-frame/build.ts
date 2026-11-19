// Bundles the game-frame benchmark with one game for one JavaScript engine:
//
//   bun bench/game-frame/build.ts <project | entry.ts> <out.js> node|hermes
//
// `node`: an ES module for node or bun that loads the napi addon (node.ts).
// `hermes`: one classic script for the Hermes shell (hermes.ts), the
// engine's runtime included, no module syntax; hermesc compiles it to bytecode.

import { resolve } from "node:path";
import { bundle } from "../../src/hermes/package.ts";

const [project, out, target] = process.argv.slice(2);
if (!project || !out || (target !== "node" && target !== "hermes")) {
  process.stderr.write("usage: bun bench/game-frame/build.ts <project | entry.ts> <out.js> node|hermes\n");
  process.exit(2);
}
const game = project.endsWith(".ts") ? resolve(project) : resolve(project, "src/main.ts");
try {
  await bundle(game, resolve(import.meta.dir, `${target}.ts`), target, out);
} catch (e) {
  process.stderr.write(`${e instanceof Error ? e.message : String(e)}\n`);
  process.exit(1);
}
