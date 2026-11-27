// Prepare a `kind vfx` effect pack for the asset library (its contract:
// docs/vfx-packs.md).
//
//   bun scripts/vfx-pack.ts manifest <pack> --spec <spec.json> [--revision SHA]
//   bun scripts/vfx-pack.ts preview <pack> [--og target/debug/og] [--stage dusk|day] [--cover <content/effect.ts>] [--cover-frame 0..5] [--only <content/a.ts,content/b.ts>] [--keep] [--skip-failed]
//
// `manifest` writes the pack's assets.json (every installed file with its
// sha256 and provenance, every top-level effect with its timing) and the
// catalog fields of extension.json (minEngine, tags, keywords) from a spec:
//
//   {
//     "minEngine": "0.8.0",
//     "style": "pixel", "dim": "2d", "keywords": ["pixel"],
//     "files": [                      // first match wins, globs over pack paths
//       {"glob": "*.json", "licence": "custom", "licenceText": "...",
//        "source": {"url": "https://..."}, "credit": "...", "derived": "copied"}
//     ],
//     "effects": {"content/<path>.ts": {"title": "...", "category": "fire", "element": "fire",
//                                 "keywords": ["..."]}}
//   }
//
// `preview` installs the pack into a scratch game and has `og vfx preview`
// (crates/og-cli/src/commands/vfx_preview.rs, which owns the stage, the
// camera, the shots, the timing and the size) play the pack's effects there,
// then writes, per effect, preview/<effect>.png (six 960x540 frames in time
// order, the impact frame among them, 3 x 2, the camera placed by the
// effect's measured core; on the dusk stage, or with `--stage day` on the
// day stage, which lit effects such as smoke, dust and debris read on:
// `--stage day --only <those>` after the dusk run re-renders them alone),
// preview/<effect>.cost.json (its peak GPU milliseconds over an empty frame,
// its draws, particle slots and measured bounds) and preview/cover.png, and
// its timing back into assets.json's effects:
//
// 1. the shots, `og vfx preview --no-cost` for the pack's effects in one
//    call (the command runs them a few at a time itself), holding one GPU
//    slot (scripts/lock.sh's og_gpu_acquire) for it;
// 2. the cost, `og vfx preview --cost-only` per effect, holding every slot
//    (og_gpu_acquire_all) for that call only.
//
// The script takes its locks itself, each for one call, so a pack never holds
// the GPU across a series and other lanes' GPU work runs between its
// effects. Run it without a lock of your own.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { deflateSync, inflateSync } from "node:zlib";

const ROOT = resolve(import.meta.dir, "..");
const LOCK_SH = join(ROOT, "scripts", "lock.sh");

type Obj = Record<string, any>;

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}
const readJson = (path: string): Obj => JSON.parse(readFileSync(path, "utf8"));
/** Indented JSON, every character past ASCII escaped (as the packs' files
 * have always been written). */
const writeJson = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2).replace(/[\u0080-\uffff]/g, (c) => `\\u${c.charCodeAt(0).toString(16).padStart(4, "0")}`) + "\n");
const sha256 = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const sortedSet = (values: Iterable<string>) => [...new Set(values)].sort();

/** fnmatch: `*` and `?` match any character, `/` included; `[...]` a class. */
function fnmatch(name: string, glob: string): boolean {
  let re = "";
  for (let i = 0; i < glob.length; i++) {
    const c = glob[i];
    if (c === "*") re += ".*";
    else if (c === "?") re += ".";
    else if (c === "[" && glob.indexOf("]", i + 2) > 0) {
      const j = glob.indexOf("]", i + 2);
      const body = glob.slice(i + 1, j);
      re += "[" + (body[0] === "!" ? "^" + body.slice(1) : body).replace(/\\/g, "\\\\") + "]";
      i = j;
    } else re += c.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&");
  }
  return new RegExp(`^${re}$`, "s").test(name);
}

/** Every path under `dir`, relative and `/`-separated, in path order
 * (component by component, as a sorted walk lists them). */
function walk(dir: string): string[] {
  const out: string[] = [];
  const visit = (rel: string) => {
    for (const name of readdirSync(join(dir, rel)).sort()) {
      const path = rel ? `${rel}/${name}` : name;
      out.push(path);
      const st = lstatSync(join(dir, path));
      if (st.isDirectory() && !st.isSymbolicLink()) visit(path);
    }
  };
  visit("");
  return out;
}

/** Spawnable checked declarations; emitters and materials are dependencies. */
function effectsOf(pack: string): Map<string, string> {
  const found = new Map<string, string>();
  for (const rel of walk(pack)) {
    if (rel.startsWith("content/") && rel.endsWith(".ts")) {
      const kind = /\bfile\.(vfx2d|vfx)\s*\(/.exec(readFileSync(join(pack, rel), "utf8"))?.[1];
      if (kind) found.set(rel, kind);
    }
  }
  return found;
}

function installedFiles(pack: string): string[] {
  const out: string[] = [];
  for (const rel of walk(pack)) {
    const st = lstatSync(join(pack, rel));
    if (st.isSymbolicLink()) fail(`${rel}: symlinks are refused`);
    if (!st.isFile() || rel === "assets.json" || rel.startsWith("preview/")) continue;
    if (rel.split("/").some((part) => part.startsWith("."))) continue;
    out.push(rel);
  }
  return out;
}

function gitHead(): string {
  return spawnSync("git", ["-C", ROOT, "rev-parse", "HEAD"], { encoding: "utf8" }).stdout.trim();
}

function manifest(packArg: string, specPath: string, revision?: string) {
  const pack = packArg;
  const spec = readJson(specPath);
  const ext = readJson(join(pack, "extension.json"));
  const effects = effectsOf(pack);
  const meta: Obj = spec.effects ?? {};
  const metas = Object.values(meta) as Obj[];
  ext.minEngine = spec.minEngine;
  ext.tags = {
    style: spec.style,
    dim: spec.dim,
    category: sortedSet(metas.map((m) => m.category).filter(Boolean)),
    element: sortedSet(metas.map((m) => m.element).filter(Boolean)),
  };
  ext.keywords = sortedSet((spec.keywords ?? []).map((k: string) => k.toLowerCase()));
  writeJson(join(pack, "extension.json"), ext);

  const files = installedFiles(pack).map((rel) => {
    const rule = (spec.files as Obj[]).find((r) => fnmatch(rel, r.glob));
    if (!rule) fail(`${rel}: no provenance rule in the spec`);
    const { glob: _, ...provenance } = rule;
    return { path: rel, sha256: sha256(join(pack, rel)), ...provenance };
  });
  const old = new Map<string, Obj>();
  if (existsSync(join(pack, "assets.json"))) {
    for (const e of readJson(join(pack, "assets.json")).effects ?? []) old.set(e.path, e);
  }
  const outEffects = [...effects].map(([rel, t]) => {
    const m: Obj = meta[rel] ?? {};
    const was = old.get(rel) ?? {};
    const stem = basename(rel).replace(/\.[^.]*$/, "");
    const e: Obj = {
      path: rel,
      type: t,
      title: m.title || stem.replace(/[-_]/g, " ").replace(/\w\S*/g, (w) => w[0].toUpperCase() + w.slice(1).toLowerCase()),
      // Timing is measured by `preview`; kept from an earlier run.
      loop: was.loop ?? false,
      durationSeconds: was.durationSeconds ?? 0,
      impact: was.impact ?? false,
      category: m.category ?? "",
      keywords: sortedSet((m.keywords ?? []).map((k: string) => k.toLowerCase())),
    };
    // Optional in the contract: omitted, never empty.
    if (m.element) e.element = m.element;
    return e;
  });
  const doc = {
    format: 1,
    engine: { version: spec.minEngine, revision: revision || gitHead() },
    files,
    effects: outEffects,
  };
  writeJson(join(pack, "assets.json"), doc);
  console.log(`${pack}: ${files.length} files, ${outEffects.length} effects`);
}

function scratchGame(pack: string, og: string): { root: string; game: string; name: string } {
  const root = mkdtempSync(join(tmpdir(), "vfx-preview-"));
  const game = join(root, "game");
  const ext = readJson(join(pack, "extension.json"));
  // A vfx2d effect plays only in a 2D project.
  const form = ext.tags?.dim === "2d" ? ["--form", "2d"] : [];
  // create runs og itself (og i18n tables): the og under review, not one on PATH.
  const made = spawnSync("bun", [join(ROOT, "packages/create/bin/create.ts"), game, ...form], { encoding: "utf8", env: { ...process.env, OG_BIN: og } });
  if (made.status !== 0) fail(made.stdout + made.stderr);
  mkdirSync(join(game, "assets"), { recursive: true });
  const r = spawnSync(og, ["vfx", "add", pack, game, "--any-engine"], { encoding: "utf8" });
  if (r.status !== 0) fail(r.stdout + r.stderr);
  return { root, game, name: ext.name };
}

/** One `og vfx preview` of `ids`, holding one GPU slot (`""`,
 * og_gpu_acquire) or every slot (`"_all"`, og_gpu_acquire_all) for this
 * call only: scripts/lock.sh's own functions, taken by the shell that runs
 * og and released by its exit trap when og ends. Its error text, or null. */
function ogPreview(og: string, game: string, out: string, ids: string[], flags: string[], suffix: "" | "_all", label: string): string | null {
  const r = spawnSync("bash", [
    "-c", `. "$0" && og_gpu_acquire${suffix} "$1" && shift && "$@"`, LOCK_SH, label,
    og, "vfx", "preview", ...ids, "--project", game, "--out", out, "--json", ...flags,
  ], { encoding: "utf8", timeout: 3600_000, maxBuffer: 1 << 28 });
  return r.status === 0 ? null : (r.stdout ?? "").slice(-2000) + (r.stderr ?? "").slice(-2000);
}

function deviceName(): string {
  if (process.platform === "darwin") {
    const cpu = spawnSync("sysctl", ["-n", "machdep.cpu.brand_string"], { encoding: "utf8" }).stdout.trim();
    return `${cpu} GPU (Metal)`.trim();
  }
  return spawnSync("uname", ["-p"], { encoding: "utf8" }).stdout?.trim() || "unknown";
}

// ---------- PNG, for the cover: 8-bit RGB or RGBA in, RGB out ----------

const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

/** An 8-bit, non-interlaced RGB or RGBA PNG (what `og vfx preview` writes)
 * as RGB rows. */
function readPngRgb(path: string): { w: number; h: number; rgb: Buffer } {
  const png = readFileSync(path);
  if (!png.subarray(0, 8).equals(PNG_SIGNATURE)) fail(`${path}: not a PNG`);
  let w = 0, h = 0, channels = 0;
  const idat: Buffer[] = [];
  for (let at = 8; at < png.length;) {
    const len = png.readUInt32BE(at);
    const type = png.toString("latin1", at + 4, at + 8);
    const body = png.subarray(at + 8, at + 8 + len);
    if (type === "IHDR") {
      w = body.readUInt32BE(0);
      h = body.readUInt32BE(4);
      const [depth, color, , , interlace] = body.subarray(8, 13);
      channels = color === 2 ? 3 : color === 6 ? 4 : 0;
      if (depth !== 8 || !channels || interlace) fail(`${path}: only 8-bit non-interlaced RGB or RGBA`);
    } else if (type === "IDAT") idat.push(body);
    at += 12 + len;
  }
  const raw = inflateSync(Buffer.concat(idat));
  const stride = w * channels;
  const rows = Buffer.alloc(h * stride);
  for (let y = 0; y < h; y++) {
    const filter = raw[y * (stride + 1)];
    const src = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? rows[y * stride + x - channels] : 0;
      const b = y > 0 ? rows[(y - 1) * stride + x] : 0;
      const c = x >= channels && y > 0 ? rows[(y - 1) * stride + x - channels] : 0;
      const p = a + b - c;
      const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      const pred = [0, a, b, (a + b) >> 1, pa <= pb && pa <= pc ? a : pb <= pc ? b : c][filter];
      rows[y * stride + x] = (src[x] + pred) & 255;
    }
  }
  if (channels === 3) return { w, h, rgb: rows };
  const rgb = Buffer.alloc(w * h * 3);
  for (let i = 0; i < w * h; i++) rows.copy(rgb, i * 3, i * 4, i * 4 + 3);
  return { w, h, rgb };
}

const CRC = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function chunk(type: string, body: Buffer): Buffer {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(body.length, 0);
  head.write(type, 4, "latin1");
  let crc = 0xffffffff;
  for (const byte of [...head.subarray(4), ...body]) crc = CRC[(crc ^ byte) & 255] ^ (crc >>> 8);
  const tail = Buffer.alloc(4);
  tail.writeUInt32BE((crc ^ 0xffffffff) >>> 0, 0);
  return Buffer.concat([head, body, tail]);
}

/** The `(x, y, w, h)` part of an RGB image as an RGB PNG. */
function writePngRgbCrop(path: string, img: { w: number; rgb: Buffer }, x: number, y: number, w: number, h: number) {
  const raw = Buffer.alloc(h * (w * 3 + 1));
  for (let row = 0; row < h; row++) {
    const at = row * (w * 3 + 1);
    raw[at] = 0;
    img.rgb.copy(raw, at + 1, ((y + row) * img.w + x) * 3, ((y + row) * img.w + x + w) * 3);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr.set([8, 2, 0, 0, 0], 8);
  writeFileSync(path, Buffer.concat([PNG_SIGNATURE, chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", Buffer.alloc(0))]));
}

interface PreviewArgs {
  pack: string;
  og: string;
  stage: "dusk" | "day";
  cover?: string;
  coverFrame?: number;
  keep: boolean;
  only?: string;
  skipFailed: boolean;
}

function preview(args: PreviewArgs) {
  const pack = resolve(args.pack);
  const og = resolve(args.og);
  const assets = readJson(join(pack, "assets.json"));
  let effects: string[] = assets.effects.map((e: Obj) => e.path);
  if (args.only) {
    // Re-render these effects alone; the pack's other sheets stay.
    const only = new Set(args.only.split(","));
    const unknown = [...only].filter((e) => !effects.includes(e)).sort();
    if (unknown.length) fail(`not effects of the pack: ${unknown.join(", ")}`);
    effects = effects.filter((e) => only.has(e));
  }
  const { root, game, name } = scratchGame(pack, og);
  const work = join(root, "out");
  const id = (rel: string) => `base/vfx/${name}/${rel.slice("content/".length, -3)}`;
  const stage = ["--stage", args.stage];
  try {
    // The shots, the pack's effects in one call (the command runs them a
    // few at a time itself); a call that fails is taken apart to find the
    // effect that does not play.
    const failed = new Map<string, string>();
    if (ogPreview(og, game, work, effects.map(id), ["--no-cost", ...stage], "", `vfx-pack preview ${name}: shots, ${effects.length} effects`) !== null) {
      for (const rel of effects) {
        const err = ogPreview(og, game, work, [id(rel)], ["--no-cost", ...stage], "", `vfx-pack preview ${name}/${rel}`);
        if (err !== null) failed.set(rel, err);
      }
    }
    if (failed.size && !args.skipFailed) fail([...failed].map(([rel, err]) => `${rel}: ${err}`).join("\n"));
    if (failed.size) {
      // Reviewing a pack that is not finished: sheets for the rest.
      console.error(`do not play, skipped: ${[...failed.keys()].sort().join(", ")}`);
      effects = effects.filter((rel) => !failed.has(rel));
    }

    const device = deviceName();
    const out = join(pack, "preview");
    if (existsSync(out) && !args.only) rmSync(out, { recursive: true, force: true });
    mkdirSync(out, { recursive: true });
    const rows: [string, Obj][] = [];
    const records = new Map<string, Obj>();
    for (const rel of effects) {
      // Every GPU slot for one effect's timed run at a time.
      const err = ogPreview(og, game, work, [id(rel)], ["--cost-only", ...stage], "_all", `vfx-pack cost ${name}/${rel}`);
      if (err !== null) fail(`${rel}: ${err}`);
      const stem = rel.slice("content/".length, -3);
      const record = readJson(join(work, id(rel) + ".json"));
      records.set(rel, record);
      const target = join(out, `${stem}.png`);
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(record.sheet, target);
      const cost = {
        format: 1,
        engine: assets.engine,
        device,
        peakFrameMs: record.cost.peakFrameMs,
        peakParticles: record.peakParticles,
        draws: record.draws,
        // Measured by the preview: null for an effect that draws nothing
        // over black (and for a vfx2d).
        bounds: record.bounds,
      };
      writeJson(join(out, `${stem}.cost.json`), cost);
      rows.push([rel, cost]);
      console.log(`${rel}\tbaseline ${record.cost.baselineMs.toFixed(3)} ms\t+${cost.peakFrameMs.total.toFixed(3)} ms`);
    }

    // The cover: one frame of the named effect (else the first with an
    // impact, else the first): the one named, else its impact's when it
    // has one (an explosion's is its flash: name the fireball's).
    let cover = args.cover && records.has(args.cover) ? args.cover : undefined;
    if (!(args.only && cover === undefined)) {
      // With --only, the cover is one of the pack's effects, maybe not re-rendered.
      cover ??= effects.find((r) => records.get(r)!.impactSeconds != null) ?? effects[0];
    }
    if (cover !== undefined) {
      const record = records.get(cover)!;
      let n = 2;
      if (args.coverFrame !== undefined) n = args.coverFrame;
      else if (record.impactSeconds != null) {
        const off = (k: number) => Math.abs(record.frameSeconds[k] - record.impactSeconds);
        n = [0, 1, 2, 3, 4, 5].reduce((best, k) => (off(k) < off(best) ? k : best));
      }
      const sheet = readPngRgb(record.sheet);
      const [w, h] = [Math.floor(sheet.w / 3), Math.floor(sheet.h / 2)];
      writePngRgbCrop(join(out, "cover.png"), sheet, (n % 3) * w, Math.floor(n / 3) * h, w, h);
    }

    for (const e of assets.effects as Obj[]) {
      const record = records.get(e.path);
      if (!record) continue;
      e.loop = record.loop;
      e.durationSeconds = record.durationSeconds;
      e.impact = record.impactSeconds != null;
    }
    writeJson(join(pack, "assets.json"), assets);
    console.log(`on ${device}, each effect over its own run's empty frames`);
    for (const [rel, c] of rows) {
      const m = c.peakFrameMs;
      console.log(`${rel}\t+${m.total.toFixed(3)} ms\tsim ${m.sim.toFixed(3)}\ttransparent +${m.transparent.toFixed(3)}\tparticles ${c.peakParticles}\tdraws ${c.draws}`);
    }
  } finally {
    if (!args.keep) rmSync(root, { recursive: true, force: true });
    else console.log(`scratch kept: ${root}`);
  }
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  const usage = "usage: bun scripts/vfx-pack.ts manifest <pack> --spec <spec.json> [--revision SHA] | preview <pack> [--og <og>] [--stage dusk|day] [--cover <content/effect.ts>] [--cover-frame 0..5] [--only <content/a.ts,...>] [--keep] [--skip-failed]";
  const flags = new Map<string, string | true>();
  const positional: string[] = [];
  for (let i = 0; i < rest.length; i++) {
    const a = rest[i];
    if (!a.startsWith("--")) positional.push(a);
    else if (a === "--keep" || a === "--skip-failed") flags.set(a, true);
    else if (i + 1 < rest.length) flags.set(a, rest[++i]);
    else fail(usage);
  }
  const flag = (name: string) => flags.get(name) as string | undefined;
  if (positional.length !== 1) fail(usage);
  if (cmd === "manifest") {
    const spec = flag("--spec");
    if (!spec) fail(usage);
    return manifest(positional[0], spec, flag("--revision"));
  }
  if (cmd !== "preview") fail(usage);
  const stage = flag("--stage") ?? "dusk";
  if (stage !== "dusk" && stage !== "day") fail("--stage: dusk or day (og vfx preview --stage)");
  const frame = flag("--cover-frame");
  if (frame !== undefined && !/^[0-5]$/.test(frame)) fail("--cover-frame: 0 to 5 (default: its impact's, else the third)");
  preview({
    pack: positional[0],
    og: flag("--og") ?? join(ROOT, "target/debug/og"),
    stage,
    cover: flag("--cover"),
    coverFrame: frame === undefined ? undefined : Number(frame),
    keep: flags.has("--keep"),
    only: flag("--only"),
    skipFailed: flags.has("--skip-failed"),
  });
}

main();
