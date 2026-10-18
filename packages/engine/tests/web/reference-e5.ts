// The browser reference tiers of templates/fx-showcase's E5 stations. Each
// station's playtest (templates/fx-showcase/tests/<station>.og.ts) is a
// fresh run that shows the station at 0.5 s and takes its frame `wait`
// seconds later; each station here is the same fresh run in headless
// Chrome through reference.ts (--script, --shots), compared with
// references/<tier>/<station>.png under the same rule. The stations and
// their waits are read from the playtests, so the two cannot drift.
//
//   bun tests/web/reference-e5.ts [--webgl] [--record] [--chrome <path>]

import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number };
};

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const tests = join(pkg, "..", "..", "templates", "fx-showcase", "tests");
const pass = process.argv.slice(2);
const stations: Array<{ name: string; tick: number }> = [];
for (const f of readdirSync(tests).filter((f) => f.endsWith(".og.ts")).sort()) {
  const m = /stationPlaytest\("([a-z0-9-]+)",\s*([0-9.]+)/.exec(readFileSync(join(tests, f), "utf8"));
  if (m) stations.push({ name: m[1]!, tick: Math.round((0.5 + Number(m[2])) * 60) });
}
if (stations.length === 0) throw new Error(`no stationPlaytest in ${tests}`);
let failed = 0;
for (const s of stations) {
  const r = Bun.spawnSync(
    ["bun", join(here, "reference.ts"), ...pass, "--script", `0.5:station:${s.name}`, "--shots", `${s.name}@${s.tick}`],
    { cwd: pkg, stdout: "inherit", stderr: "inherit" },
  );
  if (r.exitCode !== 0) failed++;
}
console.log(`${failed === 0 ? "PASS" : "FAIL"} E5 stations (${stations.map((s) => s.name).join(", ")}): ${stations.length - failed} of ${stations.length} matched`);
process.exit(failed === 0 ? 0 : 1);
