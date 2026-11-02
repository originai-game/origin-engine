// The offline fulfiller of a seed style's art requests (docs/art.md §2.7):
// for each slot of <pack>/art-requests.json that has no art yet (or whose
// art is stale, or every slot with --all, or the named --slot), it writes a
// prompt from the art brief (packages/knowledge/scripts/brief.ts) and the
// slot's own `note`, paints it through the origin platform's image API with
// the pack's approved concept sheets as references, and writes
// <pack>/art/<part>.png and its entry in <pack>/art/provenance.json. It runs
// `og style art <pack>` before (completing the requests) and after (verifying
// and packing what arrived), so a pack is painted in one command; the og it
// runs is $OG_BIN, else the checkout's target/debug/og.
//
//   bun templates/ui-kit/tools/paint-style.ts templates/styles/fantasy [--all] [--slot <part>]… [--dry-run] [--shrink]
//
// Each source is kept trimmed to its art, its shorter side at most four
// times its cell's (`shrink`; --shrink alone applies it to art/ as it is).
//
// The requests name the look preset the brief reads (`lookPreset`, a
// templates/looks slug) and the concept sheets (`concepts`, paths from the
// repository root, read from the working tree, else from origin/ui-art; or
// `<git ref>:<path>`, such as origin/ui-art-2:tests/evidence/ui-art/…). Needs OG_PLATFORM_DEVELOPMENT_KEY. Slots paint in
// parallel; a failed slot is reported and the rest are kept.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { artBrief, presetLook, type ArtKind } from "../../../packages/knowledge/scripts/brief.ts";

const ROOT = resolve(import.meta.dir, "../../..");
const API = process.env.OG_IMAGE_API ?? "https://api.origingame.ai/v1/images/edits";
const MODEL = process.env.OG_IMAGE_MODEL ?? "gpt-image-2.5-sunburst";
/** Retries of a slot the image API refuses for an empty upstream key (429 insufficient_quota). */
const RETRIES = 3;

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
  /** With `opaque`: everything farther than this fraction of the cell from its edges is painted flat in it (a centre the model hazed). */
  flat?: number;
}

function usage(): never {
  console.error("usage: bun templates/ui-kit/tools/paint-style.ts <pack dir> [--all] [--slot <part>]… [--dry-run]");
  process.exit(2);
}

const args = process.argv.slice(2);
// --shrink: only finish the sources already in art/ (trim, shrink, `opaque`
// and `flat`), with no painting; each step leaves a finished source as it is.
const shrinkOnly = args.includes("--shrink");
const pack = args[0] ? resolve(args[0]) : usage();
const all = args.includes("--all");
const dry = args.includes("--dry-run");
const only = args.flatMap((a, i) => (a === "--slot" ? [args[i + 1] ?? usage()] : []));
const OG = process.env.OG_BIN ?? join(ROOT, "target/debug/og");
/** `og style art <pack>`: its output passed through, its failure reported. */
function styleArt(): boolean {
  try {
    execFileSync(OG, ["style", "art", pack], { stdio: "inherit" });
    return true;
  } catch {
    console.error(`og style art ${args[0]} failed (above)`);
    return false;
  }
}
styleArt();
const requests = JSON.parse(readFileSync(join(pack, "art-requests.json"), "utf8"));
const slots: Slot[] = requests.slots.filter((s: Slot) => s.hash && s.size);
const look = presetLook(requests.lookPreset ?? usage());
// The concept sheets: `<ref>:<path>` from that git ref; a path from the
// working tree, else from the art director's branch (origin/ui-art).
const concepts: string[] = requests.concepts ?? [];
const conceptBytes = (c: string): Buffer => {
  const git = (spec: string) => execFileSync("git", ["show", spec], { cwd: ROOT, maxBuffer: 1 << 26 });
  if (c.includes(":")) return git(c);
  return existsSync(join(ROOT, c)) ? readFileSync(join(ROOT, c)) : git(`origin/ui-art:${c}`);
};
const conceptName = (c: string) => basename(c.split(":").pop()!);
const conceptType = (c: string) => (c.toLowerCase().endsWith(".png") ? "image/png" : "image/jpeg");
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
  // A source opaque to its edges has nothing to trim (magick reports 0x0).
  const crop = box.startsWith("0x0") ? [] : ["-crop", box, "+repage"];
  execFileSync("magick", [file, ...crop, "-resize", `${side}x${side}^>`, "-define", "png:compression-level=9", file]);
}

/**
 * A light frame made solid: an image model paints a near-white ground as part
 * of the clear background, leaving a translucent hole. Everything inside the
 * art's outline (its pixels above half alpha, the outside flood-filled from a
 * corner) is laid over the slot's `opaque` colour; the outline keeps its edge.
 */
export function solidify(file: string, colour: string, flat?: number): void {
  execFileSync("magick", [
    file, "-bordercolor", "none", "-border", "2",
    "(", "+clone", "-alpha", "extract", "-threshold", "50%",
    "-fill", "gray50", "-draw", "color 0,0 floodfill",
    "-fill", "white", "-opaque", "black", "-fill", "black", "-opaque", "gray50", ")",
    "(", "-clone", "0", "-fill", colour, "-colorize", "100", "-alpha", "off", "-clone", "1", "-alpha", "off", "-compose", "copy_opacity", "-composite", ")",
    "-delete", "1", "+swap", "-compose", "over", "-composite",
    "-shave", "2x2", "-define", "png:compression-level=9", file,
  ]);
  if (flat === undefined) return;
  const [w, h] = execFileSync("magick", ["identify", "-format", "%w %h", file], { encoding: "utf8" }).split(" ").map(Number);
  const x0 = Math.round(w! * flat), y0 = Math.round(h! * flat);
  execFileSync("magick", [file, "-fill", colour, "-draw", `rectangle ${x0},${y0} ${w! - 1 - x0},${h! - 1 - y0}`, "-define", "png:compression-level=9", file]);
}

/**
 * A source painted on an opaque ground (the image model sometimes ignores the
 * transparent background for a light frame): when all four corners are
 * opaque, the ground is flood-filled clear from each corner, within a small
 * fuzz, up to the art's outline.
 */
export function clearGround(file: string): void {
  const [w, h] = execFileSync("magick", ["identify", "-format", "%w %h", file], { encoding: "utf8" }).split(" ").map(Number);
  const corners = [[0, 0], [w! - 1, 0], [0, h! - 1], [w! - 1, h! - 1]];
  const alpha = corners.map(([x, y]) => Number(execFileSync("magick", [file, "-format", `%[fx:p{${x},${y}}.a]`, "info:"], { encoding: "utf8" })));
  if (alpha.some((a) => a < 0.5)) return;
  execFileSync("magick", [file, "-alpha", "set", "-fuzz", "20%", "-fill", "none", ...corners.flatMap(([x, y]) => ["-draw", `color ${x},${y} floodfill`]), file]);
  console.log(`${file}: its opaque ground cleared from the corners`);
}

/** What paint() does to a source after it arrives: its ground cleared, trimmed and shrunk, made solid where the slot says. */
function finish(file: string, s: Slot): void {
  clearGround(file);
  shrink(file, s);
  if (s.opaque) solidify(file, s.opaque, s.flat);
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
  concepts.forEach((c, i) => form.append("image[]", new Blob([conceptData[i]!], { type: conceptType(c) }), conceptName(c)));
  const started = Date.now();
  // The platform spreads requests over several upstream keys, one of which
  // may be out of credit (2026-11-02): a 429 insufficient_quota is tried
  // again, up to RETRIES times with a growing pause, and logged.
  let res: Response;
  let body: any;
  for (let attempt = 0; ; attempt++) {
    res = await fetch(API, { method: "POST", headers: { Authorization: `Bearer ${process.env.OG_PLATFORM_DEVELOPMENT_KEY}` }, body: form });
    body = await res.json().catch(() => null);
    const quota = res.status === 429 && body?.error?.type === "insufficient_quota";
    if (!quota || attempt === RETRIES) break;
    const wait = 2000 * 2 ** attempt;
    console.log(`${s.name}: 429 ${body.error.code ?? "insufficient_quota"}; retry ${attempt + 1} of ${RETRIES} in ${wait / 1000} s`);
    await new Promise((r) => setTimeout(r, wait));
  }
  const b64 = body?.data?.[0]?.b64_json;
  if (!res.ok || !b64) throw new Error(`${s.name}: ${res.status} ${JSON.stringify(body)?.slice(0, 300)}`);
  const file = join(artDir, `${part(s)}.png`);
  writeFileSync(file, Buffer.from(b64, "base64"));
  finish(file, s);
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
  for (const s of slots) {
    const file = join(artDir, `${part(s)}.png`);
    if (!existsSync(file)) continue;
    finish(file, s);
    // A slot whose prompt is the one it was painted from changed only in
    // how it is finished (`opaque`, `flat`): finished again, it fulfils
    // its request as it stands.
    const p = provenance[s.name];
    if (p && p.request !== s.hash && p.prompt === prompt(s)) {
      p.request = s.hash;
      p.finished = { opaque: s.opaque ?? null, flat: s.flat ?? null };
      console.log(`${s.name}: finished again for its request`);
    }
  }
  writeFileSync(provPath, JSON.stringify(Object.fromEntries(Object.entries(provenance).sort()), null, 2) + "\n");
  console.log("finished the sources in", artDir);
  process.exit(styleArt() ? 0 : 1);
}
if (!process.env.OG_PLATFORM_DEVELOPMENT_KEY && !dry) throw new Error("OG_PLATFORM_DEVELOPMENT_KEY is not set");
const results = await Promise.allSettled(todo.map(paint));
for (const r of results) if (r.status === "rejected") console.error(String(r.reason));
if (!dry) writeFileSync(provPath, JSON.stringify(Object.fromEntries(Object.entries(provenance).sort()), null, 2) + "\n");
console.log(`${todo.length - results.filter((r) => r.status === "rejected").length} of ${todo.length} slots ${dry ? "drafted" : "painted"}`);
if (!dry && !styleArt()) process.exit(1);
