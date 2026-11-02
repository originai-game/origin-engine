// The offline fulfiller of a seed style's art requests (docs/art.md §2.7):
// for each slot of <pack>/art-requests.json that has no art yet (or whose
// art is stale, or every slot with --all, or the named --slot), it writes a
// prompt from the art brief (packages/knowledge/scripts/brief.ts) and the
// slot's own `note`, paints it through the origin platform's image API with
// the pack's approved concept sheets as references, and writes
// <pack>/art/<part>.png and its entry in <pack>/art/provenance.json. Then
// `og style art <pack>` verifies and packs what arrived.
//
//   bun templates/ui-kit/tools/paint-style.ts templates/styles/fantasy [--all] [--slot <part>]… [--dry-run] [--shrink]
//
// Each source is kept trimmed to its art, its shorter side at most four
// times its cell's (`shrink`; --shrink alone applies it to art/ as it is).
//
// The requests name the look preset the brief reads (`lookPreset`, a
// templates/looks slug) and the concept sheets (`concepts`, paths from the
// repository root). Needs OG_PLATFORM_DEVELOPMENT_KEY. Slots paint in
// parallel; a failed slot is reported and the rest are kept.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { artBrief, presetLook, type ArtKind } from "../../../packages/knowledge/scripts/brief.ts";

const ROOT = resolve(import.meta.dir, "../../..");
const API = process.env.OG_IMAGE_API ?? "https://api.origingame.ai/v1/images/edits";
const MODEL = process.env.OG_IMAGE_MODEL ?? "gpt-image-2.5-sunburst";

interface Slot {
  name: string;
  kind: string;
  size: [number, number];
  insets?: [number, number, number, number];
  states: string[];
  hash: string;
  note?: string;
  /** A ground colour (`#rrggbb`): everything inside the art's outline is made opaque on it. */
  opaque?: string;
}

function usage(): never {
  console.error("usage: bun templates/ui-kit/tools/paint-style.ts <pack dir> [--all] [--slot <part>]… [--dry-run]");
  process.exit(2);
}

const args = process.argv.slice(2);
// --shrink: only trim and shrink the sources already in art/ (no painting).
const shrinkOnly = args.includes("--shrink");
const pack = args[0] ? resolve(args[0]) : usage();
const all = args.includes("--all");
const dry = args.includes("--dry-run");
const only = args.flatMap((a, i) => (a === "--slot" ? [args[i + 1] ?? usage()] : []));
const requests = JSON.parse(readFileSync(join(pack, "art-requests.json"), "utf8"));
const slots: Slot[] = requests.slots.filter((s: Slot) => s.hash && s.size);
const look = presetLook(requests.lookPreset ?? usage());
// The concept sheets: from the working tree, else from the art director's branch (origin/ui-art).
const concepts: string[] = requests.concepts ?? [];
const conceptBytes = (c: string): Buffer =>
  existsSync(join(ROOT, c)) ? readFileSync(join(ROOT, c)) : execFileSync("git", ["show", `origin/ui-art:${c}`], { cwd: ROOT, maxBuffer: 1 << 26 });
const conceptData = concepts.map(conceptBytes);
const artDir = join(pack, "art");
mkdirSync(artDir, { recursive: true });
const provPath = join(artDir, "provenance.json");
const provenance: Record<string, Record<string, unknown>> = existsSync(provPath) ? JSON.parse(readFileSync(provPath, "utf8")) : {};

const part = (s: Slot) => s.name.split("/").pop()!;
const todo = slots.filter((s) => {
  if (only.length > 0) return only.includes(part(s));
  return all || !existsSync(join(artDir, `${part(s)}.png`)) || provenance[s.name]?.request !== s.hash;
});

/** The slot's prompt: the brief's style, palette and rules, one cell at the slot's own aspect. */
function prompt(s: Slot): string {
  const [w, h] = s.size;
  const kind = s.kind as ArtKind;
  const subject = `the ${part(s).replace(/-/g, " ")} of the ${requests.style} UI style for ${requests.premise}`;
  const brief = artBrief(look, kind, subject, s.insets ? { insets: s.insets } : {});
  // The brief's sheet of state cells gives way to one cell: its states are derived on packing.
  const lines = brief.prompt.split("\n").filter((l) => !/^(Size|Aspect|States):/.test(l));
  const ratio = w / h;
  lines.push(
    `Shape: one single ${part(s)} only, centred, filling the image ${ratio >= 1 ? "across its width" : "down its height"}, ${ratio.toFixed(2)} times as wide as it is tall (${w} x ${h} proportions); nothing else in the image.`,
  );
  if (s.note) lines.push(`Art direction: ${s.note}`);
  if (concepts.length > 0) lines.push("The reference images are the approved concept of this style's screens: paint this piece as it appears there, in their exact material, colour, ornament and finish; do not copy their layout or text.");
  return lines.join("\n");
}

function size(s: Slot): string {
  const r = s.size[0] / s.size[1];
  return r > 1.25 ? "1536x1024" : r < 0.8 ? "1024x1536" : "1024x1024";
}

/**
 * The source as the pack keeps it: trimmed to its art and no larger than it
 * needs, its shorter side at most four times the cell's (the 3× atlas with
 * headroom), so a pack's sources stay a few hundred kilobytes.
 */
export function shrink(file: string, s: Slot): void {
  const side = 4 * Math.min(s.size[0], s.size[1]);
  // The art's box as og style art finds it (alpha above 8 of 255): a generated
  // image's ground carries faint alpha noise that a plain trim keeps.
  const box = execFileSync("magick", [file, "-alpha", "extract", "-threshold", "3.2%", "-format", "%@", "info:"], { encoding: "utf8" }).trim();
  if (!/^\d+x\d+\+\d+\+\d+$/.test(box)) throw new Error(`${file}: no art to trim (${box})`);
  execFileSync("magick", [file, "-crop", box, "+repage", "-resize", `${side}x${side}^>`, "-define", "png:compression-level=9", file]);
}

/**
 * A light frame made solid: an image model paints a near-white ground as part
 * of the clear background, leaving a translucent hole. Everything inside the
 * art's outline (its pixels above half alpha, the outside flood-filled from a
 * corner) is laid over the slot's `opaque` colour; the outline keeps its edge.
 */
export function solidify(file: string, colour: string): void {
  execFileSync("magick", [
    file, "-bordercolor", "none", "-border", "2",
    "(", "+clone", "-alpha", "extract", "-threshold", "50%",
    "-fill", "gray50", "-draw", "color 0,0 floodfill",
    "-fill", "white", "-opaque", "black", "-fill", "black", "-opaque", "gray50", ")",
    "(", "-clone", "0", "-fill", colour, "-colorize", "100", "-alpha", "off", "-clone", "1", "-compose", "copy_opacity", "-composite", ")",
    "-delete", "1", "+swap", "-compose", "over", "-composite",
    "-shave", "2x2", "-define", "png:compression-level=9", file,
  ]);
}

async function paint(s: Slot): Promise<void> {
  const text = prompt(s);
  if (dry) {
    console.log(`--- ${s.name} (${size(s)})\n${text}\n`);
    return;
  }
  const form = new FormData();
  form.set("model", MODEL);
  form.set("background", "transparent");
  form.set("size", size(s));
  form.set("prompt", text);
  concepts.forEach((c, i) => form.append("image[]", new Blob([conceptData[i]!], { type: "image/jpeg" }), basename(c)));
  const started = Date.now();
  const res = await fetch(API, { method: "POST", headers: { Authorization: `Bearer ${process.env.OG_PLATFORM_DEVELOPMENT_KEY}` }, body: form });
  const body = await res.json().catch(() => null);
  const b64 = body?.data?.[0]?.b64_json;
  if (!res.ok || !b64) throw new Error(`${s.name}: ${res.status} ${JSON.stringify(body)?.slice(0, 300)}`);
  const file = join(artDir, `${part(s)}.png`);
  writeFileSync(file, Buffer.from(b64, "base64"));
  shrink(file, s);
  if (s.opaque) solidify(file, s.opaque);
  provenance[s.name] = {
    model: MODEL,
    prompt: text,
    request: s.hash,
    references: concepts,
    generation: body.id ?? body.created ?? null,
    date: new Date().toISOString().slice(0, 10),
  };
  console.log(`${s.name}: painted in ${Math.round((Date.now() - started) / 1000)} s`);
}

if (shrinkOnly) {
  for (const s of slots) if (existsSync(join(artDir, `${part(s)}.png`))) shrink(join(artDir, `${part(s)}.png`), s);
  console.log("shrunk the sources in", artDir);
  process.exit(0);
}
if (!process.env.OG_PLATFORM_DEVELOPMENT_KEY && !dry) throw new Error("OG_PLATFORM_DEVELOPMENT_KEY is not set");
const results = await Promise.allSettled(todo.map(paint));
for (const r of results) if (r.status === "rejected") console.error(String(r.reason));
if (!dry) writeFileSync(provPath, JSON.stringify(Object.fromEntries(Object.entries(provenance).sort()), null, 2) + "\n");
console.log(`${todo.length - results.filter((r) => r.status === "rejected").length} of ${todo.length} slots painted; now: og style art ${args[0]}`);
