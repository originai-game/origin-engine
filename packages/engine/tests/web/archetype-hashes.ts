// The three archetypes' world hashes on the native host and in Chrome: each
// archetype's replay playtest script (without its dumps and quit) runs in
// headless Chrome through tests/web/template.ts, advancing past the script's
// last step, and in `og run --headless` at the same play surface (the
// page's canvas size, which the archetypes read at start through
// og.render.capabilities), shifted by the ticks the page took before the
// probe scheduled it; the two dump_world hashes at the same tick must be
// equal. Writes the table to --out (default target/evidence/archetype-hashes.md).
//
//   bun tests/web/archetype-hashes.ts [--webgl] [--out <file.md>] [--chrome <path>]

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { bundleGame } from "../../src/node/build.ts";
import { ogBin } from "../support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const outFile = opt("--out") ?? join(repo, "target", "evidence", "archetype-hashes.md");
const decode = (b: Uint8Array) => new TextDecoder().decode(b);

interface Row { archetype: string; surface: string; tick: number; web: string; native: string; same: boolean }
const rows: Row[] = [];
for (const archetype of ["runner", "match3", "topdown"]) {
  const project = join(repo, "templates", "archetypes", archetype);
  // The playtest module, bundled as og test does so `@origingame/engine/playtest` resolves.
  const bundled = join(repo, "target", "tmp", `archetype-hashes-${archetype}.js`);
  const b = await bundleGame(join(project, "tests", "replay.og.ts"), bundled, "native");
  if (!b.ok) throw new Error(b.reason);
  const replay = (await import(bundled)) as { default: { commands: string } };
  const steps = replay.default.commands.split(";").filter((s) => !/^[\d.]+:(quit|dump_entity|dump_world|screenshot)\b/.test(s));
  const last = Math.max(...steps.map((s) => parseFloat(s)));
  const advance = Math.round(last * 60) + 30;
  const web = Bun.spawnSync(
    ["bun", "tests/web/template.ts", "--project", project, "--commands", steps.join(";"), "--advance", String(advance), ...(args.includes("--webgl") ? ["--webgl"] : []), ...(opt("--chrome") ? ["--chrome", opt("--chrome")!] : [])],
    { cwd: pkg, stdout: "pipe", stderr: "pipe" },
  );
  if (web.exitCode !== 0) throw new Error(`${archetype} in Chrome: ${decode(web.stderr)}${decode(web.stdout).slice(-2000)}`);
  const answer = JSON.parse(decode(web.stdout)) as { capture: { width: number; height: number }; dump: { tick: number; hash: string } };
  const { width, height } = answer.capture;
  const start = answer.dump.tick - advance;
  // The same steps from tick 0, shifted by the ticks the page ran before scheduling them.
  const shifted = steps.map((s) => {
    const at = s.indexOf(":");
    return `${(parseFloat(s.slice(0, at)) + start / 60).toFixed(4)}${s.slice(at)}`;
  });
  const end = (answer.dump.tick / 60).toFixed(4);
  const dump = `archetype-hash-${archetype}.json`;
  const native = Bun.spawnSync(
    [ogBin(), "run", project, "--headless", "--size", `${width}x${height}`, "--commands", [...shifted, `${end}:dump_world:${dump}`, `${(answer.dump.tick / 60 + 0.05).toFixed(4)}:quit`].join(";")],
    { cwd: repo, stdout: "pipe", stderr: "pipe" },
  );
  if (native.exitCode !== 0) throw new Error(`${archetype} natively: ${decode(native.stderr)}`);
  const d = JSON.parse(readFileSync(join(project, "og-out", dump), "utf8")) as { tick: number; hash: string };
  rows.push({ archetype, surface: `${width}x${height}`, tick: d.tick, web: answer.dump.hash, native: d.hash, same: d.tick === answer.dump.tick && d.hash === answer.dump.hash });
}
const table = [
  `| Archetype | Surface | Tick | Chrome (${args.includes("--webgl") ? "WebGL2" : "WebGPU"}) | Native | Equal |`,
  "|---|---|---|---|---|---|",
  ...rows.map((r) => `| ${r.archetype} | ${r.surface} | ${r.tick} | \`${r.web}\` | \`${r.native}\` | ${r.same ? "yes" : "**no**"} |`),
].join("\n");
mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, table + "\n");
console.log(table);
process.exit(rows.every((r) => r.same) ? 0 : 1);
