// Resolves every slot of src/media.ts into public/m/ and writes
// src/generated/media.json, which the pages read.
//
//   bun scripts/media.ts            fetch what is missing, encode, write the index
//
// Stills become AVIF and WebP at 960/1920/3840 px wide (never upscaled) plus
// a JPEG fallback; films become H.264 MP4 (faststart, no audio) with a JPEG
// poster. Showcase entries come from the archive's bucket through
// scripts/showcase.ts (OG_SHOWCASE_S3_* secrets); repository files are read
// in place. Encoded files are content-named, so an unchanged source is
// never re-encoded. public/m/ is not committed: on Cloudflare it is the R2
// bucket behind PUBLIC_MEDIA_BASE.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { $ } from "bun";
import { download, readManifest } from "../../scripts/showcase.ts";
import { media, type Slot } from "../src/media.ts";

const site = join(import.meta.dir, "..");
const root = join(site, "..");
const out = join(site, "public/m");
const cache = join(root, "target/site-media");
mkdirSync(out, { recursive: true });
mkdirSync(cache, { recursive: true });
mkdirSync(join(site, "src/generated"), { recursive: true });

const WIDTHS = [960, 1920, 3840];

export interface Resolved {
  kind: "image" | "video" | "pending";
  status: Slot["status"];
  width?: number;
  height?: number;
  duration?: number;
  /** image: { avif: [[url, w]], webp: [[url, w]], jpg: url }; video: { mp4, poster } */
  avif?: [string, number][];
  webp?: [string, number][];
  jpg?: string;
  mp4?: string;
  poster?: string;
  /** Where it came from: commit, date, machine, the archive id or path. */
  from: { id?: string; path?: string; commit?: string; date?: string; machine?: string; pending?: string };
}

const entries = new Map(readManifest().map((e) => [e.id, e]));

async function sourceFile(slot: Slot): Promise<{ file: string; ext: string; from: Resolved["from"] } | null> {
  const src = slot.source;
  if ("pending" in src) return null;
  if ("repo" in src) {
    const file = join(root, src.repo);
    if (!existsSync(file)) throw new Error(`media: ${src.repo} does not exist`);
    const commit = src.commit ?? (await $`git -C ${root} log -1 --format=%h -- ${src.repo}`.text()).trim();
    const date = (await $`git -C ${root} log -1 --format=%cs -- ${src.repo}`.text()).trim();
    return { file, ext: src.repo.split(".").pop()!, from: { path: src.repo, commit, date } };
  }
  const e = entries.get(src.showcase);
  if (!e) throw new Error(`media: no showcase entry ${src.showcase}`);
  if (e.excluded) throw new Error(`media: ${src.showcase} is excluded: ${e.reason}`);
  const ext = e.key.split(".").pop()!;
  const file = join(cache, `${e.sha256}.${ext}`);
  if (!existsSync(file)) writeFileSync(file, await download(e.key));
  return { file, ext, from: { id: e.id, commit: e.commit?.slice(0, 9), date: e.date, machine: e.machine ?? undefined } };
}

async function dims(file: string): Promise<[number, number]> {
  const [w, h] = (await $`magick identify -format %w,%h ${file}[0]`.text()).split(",").map(Number);
  return [w, h];
}

async function still(name: string, src: string, hash: string, crop?: string): Promise<Partial<Resolved>> {
  let file = src;
  if (crop) {
    file = join(cache, `${hash}-crop.png`);
    if (!existsSync(file)) await $`magick ${src}[0] -crop ${crop} +repage ${file}`.quiet();
  }
  const [w, h] = await dims(file);
  const widths = [...WIDTHS.filter((x) => x < w), Math.min(w, 3840)].filter((x, i, a) => a.indexOf(x) === i);
  const avif: [string, number][] = [];
  const webp: [string, number][] = [];
  for (const width of widths) {
    const base = `${name}-${hash}-${width}`;
    const a = join(out, `${base}.avif`);
    const wp = join(out, `${base}.webp`);
    if (!existsSync(a)) await $`magick ${file}[0] -resize ${width}x -quality 62 ${a}`.quiet();
    if (!existsSync(wp)) await $`magick ${file}[0] -resize ${width}x -quality 82 ${wp}`.quiet();
    avif.push([`${base}.avif`, width]);
    webp.push([`${base}.webp`, width]);
  }
  const jpg = `${name}-${hash}.jpg`;
  if (!existsSync(join(out, jpg))) await $`magick ${file}[0] -resize ${Math.min(w, 1920)}x -quality 84 ${join(out, jpg)}`.quiet();
  return { kind: "image", width: w, height: h, avif, webp, jpg };
}

async function film(name: string, file: string, hash: string, posterAt: number): Promise<Partial<Resolved>> {
  const probe = JSON.parse(await $`ffprobe -v error -select_streams v:0 -show_entries stream=width,height:format=duration -of json ${file}`.text());
  const width = probe.streams[0].width as number;
  const height = probe.streams[0].height as number;
  const duration = Number(probe.format.duration);
  const mp4 = `${name}-${hash}.mp4`;
  const poster = `${name}-${hash}.jpg`;
  if (!existsSync(join(out, mp4)))
    await $`ffmpeg -loglevel error -y -i ${file} -an -c:v libx264 -preset slow -crf 20 -pix_fmt yuv420p -movflags +faststart ${join(out, mp4)}`;
  if (!existsSync(join(out, poster)))
    await $`ffmpeg -loglevel error -y -ss ${Math.min(posterAt, Math.max(0, duration - 0.1))} -i ${file} -frames:v 1 -q:v 3 ${join(out, poster)}`;
  return { kind: "video", width, height, duration, mp4, poster };
}

const index: Record<string, Resolved> = {};
for (const [name, slot] of Object.entries(media) as [string, Slot][]) {
  const got = await sourceFile(slot);
  if (!got) {
    index[name] = { kind: "pending", status: "pending", from: { pending: (slot.source as { pending: string }).pending } };
    continue;
  }
  const hash = createHash("sha256").update(readFileSync(got.file)).update(slot.crop ?? "").digest("hex").slice(0, 10);
  const enc = /^(mp4|webm|mov)$/.test(got.ext) ? await film(name, got.file, hash, slot.posterAt ?? 1) : await still(name, got.file, hash, slot.crop);
  index[name] = { ...(enc as Resolved), status: slot.status, from: got.from };
  process.stdout.write(`${name}: ${enc.kind} ${enc.width}×${enc.height}${slot.status === "final" ? "" : ` (${slot.status})`}\n`);
}
writeFileSync(join(site, "src/generated/media.json"), `${JSON.stringify(index, null, 1)}\n`);
process.stdout.write(`wrote src/generated/media.json: ${Object.keys(index).length} slots\n`);
