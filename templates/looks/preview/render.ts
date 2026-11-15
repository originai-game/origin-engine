// Draws each look's preview (docs/art.md §2.6): the look's preset scene,
// cooked and run by the engine, so a preview never shows more than a game in
// that look draws. preview.json says which scene and art each look or
// variant takes; a scene is copied to target/look-previews/<name>/, its
// look set to extend the preset (and the variant), cooked and run headless
// for one frame at 1.5 s.
//
//   bun templates/looks/preview/render.ts [--out DIR] [--size WxH] [--webp] [name ...]
//
// A name is a look slug or <slug>:<variant>; none draws every look of the
// catalogue. Writes DIR/<name>.png (default target/look-previews/frames);
// with --webp the catalogue's previews/<slug>.webp too (960x540, by
// ImageMagick). Run the og and addon of the commit under test (OG_NATIVE_ADDON,
// OG_ENGINE_PACKAGE): a preset is include_str! into og-cook.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const repo = resolve(import.meta.dir, "../../..");
const looks = join(repo, "templates/looks");
const args = process.argv.slice(2);
const flag = (k: string, d: string) => {
  const i = args.indexOf(k);
  if (i < 0) return d;
  const v = args[i + 1]!;
  args.splice(i, 2);
  return v;
};
const out = resolve(flag("--out", join(repo, "target/look-previews/frames")));
const size = flag("--size", "960x540");
const og = resolve(flag("--og", join(repo, "target/debug/og")));
const webp = args.includes("--webp");
const names = args.filter((a) => !a.startsWith("--"));

interface Scene {
  scene: "2d" | "3d";
  models?: string;
  art?: string;
  sprites?: Record<string, { file: string; height: number }>;
  view?: number;
  heights?: Record<string, number>;
}
const table = JSON.parse(readFileSync(join(import.meta.dir, "preview.json"), "utf8")) as Record<string, Scene>;
const catalogue = JSON.parse(readFileSync(join(repo, "packages/knowledge/looks/catalogue.json"), "utf8")) as { styles: { slug: string }[] };
const todo = names.length ? names : catalogue.styles.map((s) => s.slug);

/** An image's width and height, from its PNG or JPEG header. */
function size2(file: string): [number, number] {
  const b = readFileSync(file);
  if (b.readUInt32BE(0) === 0x89504e47) return [b.readUInt32BE(16), b.readUInt32BE(20)];
  for (let i = 2; i < b.length; ) {
    const marker = b[i + 1]!;
    if (marker >= 0xc0 && marker <= 0xc3) return [b.readUInt16BE(i + 7), b.readUInt16BE(i + 5)];
    i += 2 + b.readUInt16BE(i + 2);
  }
  throw new Error(`${file}: no size in its header`);
}
const aspect = (file: string) => {
  const [w, h] = size2(file);
  return w / h;
};

/** The art directory's file for a beat (hero.png, ground.jpg, ...). */
function beatFile(dir: string, beat: string): string {
  const f = readdirSync(dir).find((n) => n.startsWith(`${beat}.`) && /\.(png|jpe?g)$/.test(n));
  if (!f) throw new Error(`${dir}: no ${beat}.png or .jpg`);
  return f;
}

mkdirSync(out, { recursive: true });
let failed = 0;
for (const name of todo) {
  const [slug, variant] = name.split(":") as [string, string | undefined];
  const scene: Scene = table[name] ?? table[slug] ?? { scene: "3d" };
  const dir = join(repo, "target/look-previews", name.replace(":", "@"));
  rmSync(dir, { recursive: true, force: true });
  cpSync(join(import.meta.dir, scene.scene), dir, { recursive: true });
  mkdirSync(join(dir, "node_modules/@origingame"), { recursive: true });
  symlinkSync(join(repo, "packages/engine"), join(dir, "node_modules/@origingame/engine"));
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  writeFileSync(join(dir, "assets/looks/main.ogt"), `type look\nversion 1\n---\nname "Preview: ${name}"\nextends ${JSON.stringify(slug)}\n${variant ? `variant ${JSON.stringify(variant)}\n` : ""}`);
  if (scene.models) cpSync(join(looks, scene.models), join(dir, "assets/models"), { recursive: true });
  if (scene.scene === "3d") {
    const sprites: Record<string, { path: string; width: number; height: number }> = {};
    if (scene.art) cpSync(join(looks, scene.art), join(dir, "assets/art"), { recursive: true });
    for (const [beat, s] of Object.entries(scene.sprites ?? {})) {
      sprites[beat] = { path: `art/${s.file}`, width: s.height * aspect(join(looks, scene.art!, s.file)), height: s.height };
    }
    const ground = existsSync(join(dir, "assets/models/field.ogt")) ? "models/field.ogt" : "models/ground.glb";
    writeFileSync(join(dir, "src/sprites.ts"), `// Written by templates/looks/preview/render.ts from preview.json.\nexport const GROUND = ${JSON.stringify(ground)};\nexport const SPRITES: Record<string, { path: string; width: number; height: number }> = ${JSON.stringify(sprites)};\n`);
  } else {
    const art = join(looks, scene.art!);
    cpSync(art, join(dir, "assets/art"), { recursive: true });
    // The far layer is drawn at the look's logical resolution: without its
    // own import settings the cook would conform it to a sprite's cell.
    const back = beatFile(art, "backdrop");
    if (!existsSync(join(art, `${back}.ogt`))) {
      const long = Math.max(...size2(join(art, back)));
      writeFileSync(join(dir, "assets/art", `${back}.ogt`), `type import\nversion 1\n---\n# Written by templates/looks/preview/render.ts: the far layer keeps its drawn size.\nconform {\n  size ${long}\n}\n`);
    }
    const sprites: Record<string, { file: string; width: number; height: number }> = {};
    for (const [beat, h] of Object.entries(scene.heights!)) {
      const file = beatFile(art, beat);
      sprites[beat] = { file, width: h * aspect(join(art, file)), height: h };
    }
    writeFileSync(join(dir, "src/sprites.ts"), `// Written by templates/looks/preview/render.ts from preview.json.\nexport const VIEW_HEIGHT = ${scene.view};\nexport const ART: Record<string, { file: string; width: number; height: number }> = ${JSON.stringify(sprites)};\n`);
  }
  const file = `${name.replace(":", "@")}.png`;
  rmSync(join(out, file), { force: true });
  const run = spawnSync(og, ["run", dir, "--headless", "--size", size, "--out", out, "--commands", `1.5:screenshot:${file};1.6:quit`], { encoding: "utf8", timeout: 300_000 });
  if (run.status !== 0 || !existsSync(join(out, file))) {
    failed++;
    console.log(`${name}: run failed (${run.status}): ${(run.stderr || run.stdout).trim().split("\n").slice(-4).join(" | ")}`);
    continue;
  }
  const warnings = (run.stdout + run.stderr).split("\n").filter((l) => /warning/.test(l));
  console.log(`${name}: ${join(out, file)}${warnings.length ? `\n  ${warnings.join("\n  ")}` : ""}`);
  if (webp && !variant) {
    const preview = join(repo, "packages/knowledge/previews", `${slug}.webp`);
    const m = spawnSync("magick", [join(out, file), "-resize", "960x540^", "-gravity", "center", "-extent", "960x540", "-quality", "82", "-define", "webp:method=6", preview], { encoding: "utf8" });
    if (m.status !== 0) {
      failed++;
      console.log(`${slug}: magick failed: ${m.stderr.trim()}`);
    } else console.log(`  ${preview}`);
  }
}
process.exit(failed ? 1 : 0);
