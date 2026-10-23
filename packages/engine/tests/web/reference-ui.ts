// The browser tiers of og-ui's kit frames: templates/ui-kit's reference
// playtests (tests/<playtest>.og.ts: frames-desktop, frames-phone, players,
// locales and the style-* style packs) each as the same fresh run in headless
// Chrome through reference.ts, on WebGPU (or WebGL2 with --webgl), compared
// with crates/og-ui/references/<tier>/ (each frame on its own falling back
// to metal's) under og-tests' rule: a channel off by more than 32/255 is an
// outlier; fail above 0.5 % outliers or an RGB RMSE above 0.01.
//
// Each playtest module is bundled and imported as `og test` does
// (src/node/playtest-run.ts), so its script, its size and its screenshots
// are the playtest's own: the script runs without its screenshots, `quit`
// and `dump_*`, and each screenshot at t seconds becomes a shot at tick
// round(60 t) named <prefix><file>, the reference the playtest checks it
// against (frames.ts `checkFrames`). A style pack's shots are compared with
// their cells of kit-style-<pack>[-phone].png, box-filtered as styles.ts does.
//
//   bun tests/web/reference-ui.ts [--webgl] [--record] [--only a,b] [--chrome <path>]
//
// Without --only it runs SET below (docs/ci.md §Browser reference tiers says
// which frames that is and why). --record writes this tier's references.
// Writes dist/test-reference/ui/<playtest>/<tier>/ (frames, diffs and
// reference-<tier>.json) and dist/test-reference/ui/reference-ui-<tier>.json.
// Exits 1 when a frame does not match or a run fails.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "../../src/node/build.ts";
import type { PlaytestDesc } from "../../src/playtest.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array };
};

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const kit = join(repo, "templates", "ui-kit");
const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const tier = args.includes("--webgl") ? "webgl2" : "webgpu";
const pass = args.filter((a, i) => a === "--webgl" || a === "--record" || a === "--chrome" || args[i - 1] === "--chrome");
const work = join(pkg, "dist", "test-reference", "ui");

/** The reference prefix each frames playtest checks its screenshots under (its `checkFrames` call). */
const PREFIX: Record<string, string> = {
  "frames-desktop": "kit-desktop-",
  "frames-phone": "kit-phone-",
  players: "kit-desktop-",
  locales: "kit-locale-",
};
/** A style pack's sheet per surface (styles.ts SURFACES): the box filter and the cells a row. */
const SHEETS: Record<string, { factor: number; cols: number }> = {
  "1280x720": { factor: 2, cols: 7 },
  "1170x2532": { factor: 3, cols: 10 },
};
const PACKS = ["cartoon", "fantasy", "neon", "paper", "pixel", "scifi"];
/** Every reference playtest of the kit. */
const SET = [...Object.keys(PREFIX), ...PACKS.flatMap((p) => [`style-${p}`, `style-${p}-phone`])];

const only = opt("--only")?.split(",") ?? SET;
for (const p of only) if (!SET.includes(p)) throw new Error(`--only ${p}: not a reference playtest of the kit (${SET.join(", ")})`);

const summary: Record<string, unknown> = { tier, playtests: {} as Record<string, unknown> };
let failed = 0;
const t0 = performance.now();
for (const name of only) {
  const ts = performance.now();
  const out = join(work, name);
  mkdirSync(out, { recursive: true });
  const bundled = join(out, ".playtest.js");
  const b = await bundleGame(join(kit, "tests", `${name}.og.ts`), bundled, "native");
  if (!b.ok) throw new Error(`${name}.og.ts does not bundle: ${b.reason}`);
  const { default: desc, referenceFiles }: { default: PlaytestDesc; referenceFiles?: readonly string[] } = await import(pathToFileURL(bundled).href);
  const size = desc.size ?? "1280x720";
  const pack = /^style-([a-z]+)(-phone)?$/.exec(name);
  const script: string[] = [];
  const shots: string[] = [];
  for (const step of desc.commands.split(";")) {
    const [at, action, ...rest] = step.split(":");
    if (action === "screenshot") {
      if (referenceFiles && !referenceFiles.includes(rest[0]!)) continue;
      const file = rest[0]!.replace(/\.png$/, "");
      // A style pack's sheet has no gallery (styles.ts).
      if (pack && file === "gallery") continue;
      shots.push(`${pack ? `kit-style-${pack[1]}${pack[2] ?? ""}-` : PREFIX[name]}${file}@${Math.round(Number(at) * 60)}`);
    } else if (action !== "quit" && !action!.startsWith("dump_")) script.push(step);
  }
  const sheet = pack ? SHEETS[size] : undefined;
  if (pack && !sheet) throw new Error(`${name}: no sheet layout for ${size}`);
  const cmd = ["bun", join(here, "reference.ts"), ...pass, "--project", "templates/ui-kit", "--references", "crates/og-ui/references", "--size", size, "--script", script.join(";"), "--shots", shots.join(","), "--out", out];
  if (pack) cmd.push("--sheet", `kit-style-${pack[1]}${pack[2] ?? ""}.png,${sheet!.factor},${sheet!.cols}`);
  console.log(`---- ${name} (${tier}, ${size}, ${shots.length} frames)`);
  const r = Bun.spawnSync(cmd, { cwd: pkg, stdout: "inherit", stderr: "inherit" });
  if (r.exitCode !== 0) failed++;
  (summary["playtests"] as Record<string, unknown>)[name] = { size, frames: shots.length, ok: r.exitCode === 0, ms: Math.round(performance.now() - ts), json: join(out, `reference-${tier}.json`).slice(repo.length + 1) };
}
summary["ms"] = Math.round(performance.now() - t0);
writeFileSync(join(work, `reference-ui-${tier}.json`), JSON.stringify(summary, null, 2) + "\n");
console.log(`${failed === 0 ? "PASS" : "FAIL"} og-ui kit frames on ${tier} (${only.join(", ")}): ${only.length - failed} of ${only.length} playtests matched in ${summary["ms"]} ms`);
process.exit(failed === 0 ? 0 : 1);
