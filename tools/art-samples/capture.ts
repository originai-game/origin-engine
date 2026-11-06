// The art director's per-style frames (docs/art.md §2.6): for each style of
// the catalogue, the project its family's profile draws (families.json's
// profile: templates/game-2d for 2d, templates/game for 3d; the family
// scenes.json gives the style) is copied, its look set to extend the
// style, cooked (so the cook conforms every asset to that style's art
// contract), run headless for one frame, and measured by `og style check`.
// The frames are read, not compared: nothing here is a reference.
//
//   bun tools/art-samples/capture.ts [--out DIR] [--size WxH] [--only slug|scene,...] [--mood MOOD,...] [--og PATH]
//
// Writes DIR/<slug>.png and DIR/<slug>.style.json (the check's JSON). With
// --mood, the sample's look asks each mood (docs/art.md §2.10; `own` for
// the preset's own, no mood line) and the frames are DIR/<slug>@<mood>.png.
// Run `og` and the addon of the commit under test (OG_NATIVE_ADDON,
// OG_ENGINE_PACKAGE), as every frame of a branch.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

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
/** The family each style is shown in. Internal to these tools, not in the catalogue. */
const scenes = JSON.parse(readFileSync(join(import.meta.dir, "scenes.json"), "utf8")) as Record<string, string>;
/** Each family's profile, and so the template that draws it. */
const families = JSON.parse(readFileSync(join(import.meta.dir, "families.json"), "utf8")) as Record<string, { profile: "2d" | "3d" }>;
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
  const project = join(repo, "templates", families[scene]?.profile === "2d" ? "game-2d" : "game");
  const dir = join(scratch, s.slug);
  rmSync(dir, { recursive: true, force: true });
  cpSync(project, dir, { recursive: true, filter: (p) => !/\/(cooked|og-out|node_modules|\.og)$/.test(p) });
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  for (const mood of moods.length ? moods : [""]) {
    const name = mood ? `${s.slug}@${mood}` : s.slug;
    const asked = mood && mood !== "own" ? `mood "${mood}"\n` : "";
    writeFileSync(join(dir, "assets/looks/sample.ogt"), `type look\nversion 1\n---\nname "Sample: ${name}"\nextends "${s.slug}"\n${asked}`);
    // The template's own look file names the style (GameDesc.look points at it).
    const look = readdirSync(join(dir, "assets/looks")).find((f) => f.endsWith(".ogt") && f !== "sample.ogt");
    if (look) writeFileSync(join(dir, "assets/looks", look), readFileSync(join(dir, "assets/looks/sample.ogt"), "utf8"));
    const run = spawnSync(og, ["run", dir, "--headless", "--size", size, "--out", out, "--commands", `${AT}:screenshot:${name}.png;${AT + 0.1}:quit`], { encoding: "utf8" });
    const shot = existsSync(join(out, `${name}.png`));
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
