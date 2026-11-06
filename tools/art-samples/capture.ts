// The art wave's per-style frames (docs/art.md §2.6, stream A7): for each
// style of the catalogue, its sample scene (`examples/art-<scene>`, the
// scene scenes.json gives the style) is copied, its look set to extend the
// style, cooked (so the cook conforms every asset to that style's art
// contract), run headless for one frame, and measured by `og style check`.
//
//   bun tools/art-samples/capture.ts [--out DIR] [--size WxH] [--only slug|scene,...] [--mood MOOD,...] [--og PATH]
//
// Writes DIR/<slug>.png (the reference name of the style's frame:
// examples/art-<scene>/references/<tier>/<slug>.png when DIR is that
// tier's directory) and DIR/<slug>.style.json (the check's JSON). With
// --mood, the sample's look asks each mood (docs/art.md §2.10; `own` for
// the preset's own, no mood line) and the frames are DIR/<slug>@<mood>.png.
// Run `og` and the addon of the commit under test (OG_NATIVE_ADDON,
// OG_ENGINE_PACKAGE), as every frame of a branch.
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { prepare } from "./prepare.ts";
import { recordReference } from "../../packages/engine/src/playtest.ts";

const repo = resolve(import.meta.dir, "../..");
const args = process.argv.slice(2);
const arg = (k: string, d: string) => {
  const i = args.indexOf(k);
  return i >= 0 && args[i + 1] ? args[i + 1]! : d;
};
const out = resolve(arg("--out", join(repo, "target/art-samples")));
const size = arg("--size", "1280x720");
const og = resolve(arg("--og", join(repo, "target/debug/og")));
const only = arg("--only", "").split(",").filter(Boolean);
const moods = arg("--mood", "").split(",").filter(Boolean);
/** When the frame is taken: after the first second, so loading and the camera have settled. */
const AT = 1.5;

interface Style { slug: string; look: { mood: string } }
const catalogue = JSON.parse(readFileSync(join(repo, "packages/knowledge/looks/catalogue.json"), "utf8")) as { styles: Style[] };
/** The sample scene each style is proven on: examples/art-<scene>. Internal to these tools, not in the catalogue. */
const scenes = JSON.parse(readFileSync(join(import.meta.dir, "scenes.json"), "utf8")) as Record<string, string>;
mkdirSync(out, { recursive: true });
const scratch = join(repo, "target/art-samples-runs");
let failed = 0;
for (const s of catalogue.styles) {
  const scene = scenes[s.slug];
  if (!scene) {
    console.log(`${s.slug}: no sample scene in tools/art-samples/scenes.json`);
    failed++;
    continue;
  }
  if (only.length && !only.includes(s.slug) && !only.includes(scene)) continue;
  const example = join(repo, "examples", `art-${scene}`);
  if (!existsSync(example)) {
    console.log(`${s.slug}: no sample scene ${example}`);
    failed++;
    continue;
  }
  const dir = join(scratch, s.slug);
  rmSync(dir, { recursive: true, force: true });
  cpSync(example, dir, { recursive: true, filter: (p) => !/\/(cooked|og-out|node_modules)$/.test(p) && !["references", "styles", "sets"].some((d) => p === join(example, d)) });
  // A style briefed on its own (its direction differs from the scene's base) brings its own sources: styles/<slug>/ laid over assets/.
  // A style that takes one of the scene's other asset sets names it in styles/<slug>/set: sets/<name>/ laid over the project
  // (its assets, beats.json and src/models.ts), the same set for every style that names it.
  const own = join(example, "styles", s.slug);
  const set = join(own, "set");
  if (existsSync(set)) {
    const from = join(example, "sets", readFileSync(set, "utf8").trim());
    // The set's models stand in for the scene's: the base ones are not cooked beside them.
    if (existsSync(join(from, "assets/models"))) rmSync(join(dir, "assets/models"), { recursive: true, force: true });
    cpSync(from, dir, { recursive: true });
  }
  else if (existsSync(own)) cpSync(own, join(dir, "assets"), { recursive: true });
  for (const mood of moods.length ? moods : [""]) {
    const name = mood ? `${s.slug}@${mood}` : s.slug;
    const asked = mood && mood !== "own" ? `mood "${mood}"\n` : "";
    writeFileSync(join(dir, "assets/looks/sample.ogt"), `type look\nversion 1\n---\nname "Sample: ${name}"\nextends "${s.slug}"\n${asked}`);
    writeFileSync(join(out, `${name}.briefs.txt`), prepare(dir) + "\n");
    const run = spawnSync(og, ["run", dir, "--headless", "--size", size, "--out", out, "--commands", `${AT}:screenshot:${name}.png;${AT + 0.1}:quit`], { encoding: "utf8" });
    const shot = existsSync(join(out, `${name}.png`));
    // The frame is kept as a reference: the same pixels at maximum compression, not og run's fast PNG.
    if (shot) recordReference(join(out, `${name}.png`), join(out, `${name}.png`));
    const check = spawnSync(og, ["style", "check", dir, "--no-frames", "--json"], { encoding: "utf8" });
    writeFileSync(join(out, `${name}.style.json`), check.stdout || JSON.stringify({ error: check.stderr }));
    let verdict = `exit ${check.status}`;
    try {
      const j = JSON.parse(check.stdout) as { ok?: boolean; assets?: { ok?: boolean; asset?: string; checks?: { name: string; ok: boolean; value: string; limit: string }[] }[] };
      const off = (j.assets ?? []).filter((a) => a.ok === false).map((a) => `${a.asset} (${(a.checks ?? []).filter((c) => !c.ok).map((c) => `${c.name} ${c.value} against ${c.limit}`).join("; ")})`);
      verdict += off.length ? `, off: ${off.join(" ")}` : ", every asset ok";
    } catch {
      verdict += `, ${check.stderr.trim().split("\n").pop()}`;
    }
    if (!shot || run.status !== 0) {
      failed++;
      console.log(`${name} (${scene}): run failed (${run.status}): ${(run.stderr || run.stdout).trim().split("\n").slice(-3).join(" | ")}`);
    } else console.log(`${name} (${scene}): ${join(out, name + ".png")}; style check ${verdict}`);
  }
}
process.exit(failed ? 1 : 0);
