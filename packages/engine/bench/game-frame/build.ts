// Bundles the game-frame benchmark with one game for one JavaScript engine:
//
//   bun bench/game-frame/build.ts <project | entry.ts> <out.js> node|hermes
//
// `node`: an ES module for node or bun that loads the napi addon (node.ts).
// `hermes`: one classic script for hosts/hermes (hermes.ts), the engine's
// runtime included, no module syntax; hermesc compiles it to bytecode.

import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { engineFile } from "../../src/node/build.ts";

const [project, out, target] = process.argv.slice(2);
if (!project || !out || (target !== "node" && target !== "hermes")) {
  process.stderr.write("usage: bun bench/game-frame/build.ts <project | entry.ts> <out.js> node|hermes\n");
  process.exit(2);
}
const dir = mkdtempSync(join(tmpdir(), "og-game-frame-"));
const entry = join(dir, "entry.ts");
const driver = resolve(import.meta.dir, `${target}.ts`);
const game = project.endsWith(".ts") ? resolve(project) : resolve(project, "src/main.ts");
writeFileSync(entry, `import game from ${JSON.stringify(game)};\nimport { main } from ${JSON.stringify(driver)};\nmain(game);\n`);
const r = await Bun.build({
  entrypoints: [entry],
  target: target === "node" ? "node" : "browser",
  format: target === "node" ? "esm" : "iife",
  // The addon's loader is node's; the Hermes runner has none.
  external: target === "node" ? ["*.node"] : [],
  plugins: [{
    name: "origin-engine",
    setup(build) {
      build.onResolve({ filter: /^@origingame\/engine(\/.*)?$/ }, (a) => {
        const file = engineFile(a.path);
        if (!file) throw new Error(`${a.path} is not an export of @origingame/engine`);
        return { path: file };
      });
    },
  }],
});
rmSync(dir, { recursive: true, force: true });
if (!r.success) {
  for (const l of r.logs) process.stderr.write(`${String(l)}\n`);
  process.exit(1);
}
await Bun.write(out, r.outputs[0]!);
