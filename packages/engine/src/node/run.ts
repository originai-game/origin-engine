// `og run --headless <project>` as far as M2 goes: load a game bundle on the
// native host, optionally run registered actions once before the first tick,
// advance N fixed ticks and print the world hash. The scheduled command
// channel (`--commands "t:action;..."`) is the probe's, in M5.
//
//   bun src/node/run.ts --entry <game.ts> [--ticks N] [--seed S] [--setup action[:arg...]]... [--json]

import { loadGame } from "./index.ts";

const argv = process.argv.slice(2);
let entry = "";
let ticks = 0;
let seed: number | undefined;
let asJson = false;
const setup: string[] = [];
for (let i = 0; i < argv.length; i++) {
  const a = argv[i]!;
  const next = () => {
    const v = argv[++i];
    if (v === undefined) fail(`${a} needs a value`);
    return v!;
  };
  if (a === "--entry") entry = next();
  else if (a === "--ticks") ticks = Number(next());
  else if (a === "--seed") seed = Number(next());
  else if (a === "--setup") setup.push(next());
  else if (a === "--json") asJson = true;
  else fail(`unknown argument '${a}'`);
}

function fail(reason: string): never {
  process.stderr.write(`og run: ${reason}\n`);
  process.exit(2);
}

if (!entry) fail("no game entry: pass --entry <file>");
if (!Number.isInteger(ticks) || ticks < 0) fail("--ticks takes a whole number of fixed steps");

const loaded = await loadGame(entry, seed === undefined ? {} : { seed });
if (!loaded.ok) fail(`${entry}: ${loaded.reason}`);
const engine = loaded.value;
for (const step of setup) {
  const [action, ...args] = step.split(":");
  const r = engine.command(action!, args);
  if (!r.ok) fail(`--setup ${step}: ${r.reason}`);
}
const t0 = performance.now();
const advanced = engine.advance(ticks);
const ms = performance.now() - t0;
if (!advanced.ok) fail(advanced.reason);
const dump = engine.probe.dump({ what: "world" }) as { hash: string; entities: number };
if (asJson) {
  process.stdout.write(JSON.stringify({ entry, ticks, tick: advanced.value, hash: dump.hash, entities: dump.entities, advanceMs: +ms.toFixed(2), runtime: typeof Bun !== "undefined" ? `bun ${Bun.version}` : `node ${process.version}` }) + "\n");
} else {
  process.stdout.write(`tick ${advanced.value} hash ${dump.hash} entities ${dump.entities}\n`);
}
