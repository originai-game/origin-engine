// Writes `<version>.save` and `<version>.json` beside this file: game.ts run
// on a published engine for 90 ticks, then world.snapshot. The JSON keeps
// what that engine read back from its own save, for the test to compare.
//
//   npm pack @origingame/engine@0.7.7 @origingame/engine-linux-x64@0.7.7
//   (unpack each), then
//   bun packages/engine/tests/fixtures/saves/make.ts <engine package dir> <origin-engine.node>
//
// The game file is copied beside a node_modules/@origingame/engine link to
// that package, so its import resolves to the old engine, not this checkout.
import { cpSync, mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const [pkgArg, addon] = process.argv.slice(2);
if (!pkgArg || !addon) throw new Error("usage: bun make.ts <engine package dir> <origin-engine.node>");
const pkg = resolve(pkgArg);
const here = dirname(fileURLToPath(import.meta.url));
const version = (JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")) as { version: string }).version;

const work = mkdtempSync(join(tmpdir(), "og-save-fixture-"));
mkdirSync(join(work, "node_modules", "@origingame"), { recursive: true });
symlinkSync(pkg, join(work, "node_modules", "@origingame", "engine"));
cpSync(join(here, "game.ts"), join(work, "game.ts"));

const node = await import(join(pkg, "src", "node", "index.ts"));
node.loadNativeHost(resolve(addon));
const runtime = await import(join(pkg, "src", "runtime.ts"));
const mod = await import(join(work, "game.ts"));
const game = mod.default;
const made = runtime.createEngine(game);
if (!made.ok) throw new Error(made.reason);
const engine = made.value;
engine.advance(90);
const save = engine.world.snapshot();
if (!save.ok) throw new Error(save.reason);
const world = engine.world;
const run = world.resource(mod.Run);
const gold = Array.from({ length: 12 }, (_, i) => {
  const got = world.get(world.find(`coin-${i}`), mod.Wallet);
  if (!got.ok) throw new Error(got.reason);
  return got.value.gold;
});
writeFileSync(join(here, `${version}.save`), save.value + "\n");
writeFileSync(
  join(here, `${version}.json`),
  JSON.stringify({ engine: version, tick: world.tick, hash: engine.hash(), run: run.ok ? run.value : null, gold }, null, 2) + "\n",
);
console.log(`${version}.save: ${save.value.length} characters at tick ${world.tick}`);
engine.destroy();
