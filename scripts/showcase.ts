// The showcase archive: every screenshot, film, benchmark and measurement a
// stream produces, kept for a later web showcase of the engine's features.
// The files live in the private S3 bucket origin-engine-showcase (OVHcloud
// Object Storage, region BHS),
// content-addressed by sha256 (sha256/<2 hex>/<64 hex>.<ext>, so a file is
// stored once however often it is added); git holds only
// showcase/manifest.jsonl, one entry per artifact (docs/ci.md §Showcase).
//
//   bun scripts/showcase.ts add <file> --feature U6c --title "…" [options]
//   bun scripts/showcase.ts add --kind benchmark --feature … --title … --metric name=value unit …
//   bun scripts/showcase.ts list [--feature F] [--kind K] [--json]
//   bun scripts/showcase.ts exclude <id|file|sha256> --reason "…" [--feature F --title T]
//                                         withdraws an artifact (see below)
//   bun scripts/showcase.ts verify        every live entry's key exists, at its size
//   bun scripts/showcase.ts index         writes showcase/index.html from the manifest
//   bun scripts/showcase.ts serve [--port 8780]   the index, with the bucket's files proxied
//
// add's options: --kind frame|film|before-after|benchmark|concept (default:
// film for video and gif, benchmark for metrics without a file, else frame),
// --commit (default HEAD), --date (default today, UTC), --thread (the Amp
// thread that made it, an ID or URL; default $AMP_THREAD_ID, else
// $AMP_CURRENT_THREAD_ID; add refuses an entry without one), --machine, --metric name=value unit (repeatable),
// --pair <name> --side before|after (a before-after pair: both sides share
// the pair name), --source (where the file came from), --dry-run.
//
// Withdrawn work never reaches the showcase. add refuses any file or
// --source under a rejected/ directory (where a thread keeps a concept it
// was told not to publish), and any bytes an excluded entry holds. exclude
// marks entries `excluded: true` with a `reason`, by id, or by content (a
// file or its sha256: every entry of those bytes); content not yet in the
// manifest gets an excluded entry with no key (nothing is uploaded), so a
// later add or backfill of the same bytes from anywhere is refused. index
// and serve leave excluded entries out.
//
// The bucket is reached over S3 with OG_SHOWCASE_S3_ACCESS_KEY and
// OG_SHOWCASE_S3_SECRET_KEY (a user that can put, get and list in that bucket
// alone; Amp secrets); OG_SHOWCASE_BUCKET, OG_SHOWCASE_S3_ENDPOINT and
// OG_SHOWCASE_S3_REGION override the bucket, endpoint and region. Dimensions and durations come from the PNG header,
// else ffprobe when it is installed.

import { S3Client } from "bun";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync, appendFileSync } from "node:fs";
import { basename, extname, join } from "node:path";

declare const Bun: {
  serve(o: { port: number; fetch(r: Request): Response | Promise<Response> }): { port: number };
};

export const ROOT = join(import.meta.dir, "..");
export const MANIFEST = join(ROOT, "showcase/manifest.jsonl");
const INDEX = join(ROOT, "showcase/index.html");
const CACHE = join(ROOT, "target/showcase-cache");
const BUCKET = process.env.OG_SHOWCASE_BUCKET ?? "origin-engine-showcase";
const ENDPOINT = process.env.OG_SHOWCASE_S3_ENDPOINT ?? "https://s3.bhs.io.cloud.ovh.net";
const REGION = process.env.OG_SHOWCASE_S3_REGION ?? "bhs";
const KINDS = ["frame", "film", "before-after", "benchmark", "concept"] as const;
export type Kind = (typeof KINDS)[number];

export interface Metric {
  name: string;
  /** A number, or a string for what is not one (a world hash). */
  value: number | string;
  unit: string;
}

export interface Entry {
  id: string;
  feature: string;
  title: string;
  kind: Kind;
  commit: string | null;
  date: string;
  thread: string | null;
  /** null only for an excluded entry whose bytes were never uploaded. */
  key: string | null;
  sha256: string;
  bytes: number;
  type: string;
  width?: number;
  height?: number;
  duration?: number;
  metrics?: Metric[];
  machine?: string;
  pair?: string;
  side?: "before" | "after";
  source?: string;
  /** Withdrawn: never shown, and its bytes are refused by add. */
  excluded?: true;
  reason?: string;
}

const TYPES: Record<string, string> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".gif": "image/gif",
  ".svg": "image/svg+xml",
  ".webm": "video/webm",
  ".mp4": "video/mp4",
  ".mov": "video/quicktime",
  ".json": "application/json",
  ".jsonl": "application/x-ndjson",
  ".csv": "text/csv",
  ".txt": "text/plain",
  ".md": "text/markdown",
  ".html": "text/html",
  ".log": "text/plain",
};
const VIDEO = new Set([".webm", ".mp4", ".mov", ".gif"]);

/**
 * The Amp thread an artifact came from: `given` (an ID or a thread URL), else
 * the thread Amp names in the environment (an orb sets AMP_THREAD_ID; a
 * runner's shell may set AMP_CURRENT_THREAD_ID or neither). Undefined when
 * there is none.
 */
export function threadOf(given?: string | null): string | undefined {
  const raw = given ?? process.env.AMP_THREAD_ID ?? process.env.AMP_CURRENT_THREAD_ID;
  return /T-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i.exec(raw ?? "")?.[0];
}

/** add's refusal of withdrawn work, which a backfill catches and skips. */
export class Refused extends Error {}

/** Whether a path or source lies under a rejected/ directory. */
export const rejectedPath = (p: string | undefined) => !!p && /(^|[\\/])rejected[\\/]/i.test(p);

function fail(why: string): never {
  process.stderr.write(`showcase: ${why}\n`);
  process.exit(1);
}

// ── The bucket ──────────────────────────────────────────────────────────────

let client: S3Client | undefined;

/** The bucket's S3 client, from OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY. */
function s3(): S3Client {
  if (client) return client;
  const accessKeyId = process.env.OG_SHOWCASE_S3_ACCESS_KEY;
  const secretAccessKey = process.env.OG_SHOWCASE_S3_SECRET_KEY;
  if (!accessKeyId || !secretAccessKey) fail("set OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY (the showcase bucket's S3 user; docs/ci.md §Showcase)");
  client = new S3Client({ accessKeyId, secretAccessKey, endpoint: ENDPOINT, region: REGION, bucket: BUCKET });
  return client;
}

/** Runs `f`, retrying throttling (503 SlowDown) and dropped connections with backoff, up to ~5 min. */
async function retry<T>(what: string, f: () => Promise<T>): Promise<T> {
  for (let attempt = 0; ; attempt++) {
    try {
      return await f();
    } catch (e) {
      const code = String((e as { code?: string }).code ?? "");
      if (code === "NoSuchKey" || code === "AccessDenied" || attempt >= 10) fail(`${what}: ${code || e}`);
      await new Promise((ok) => setTimeout(ok, Math.min(60_000, 1000 * 2 ** attempt)));
    }
  }
}

/** Every object in the bucket: key → size in bytes. */
export async function objects(): Promise<Map<string, number>> {
  const out = new Map<string, number>();
  let continuationToken: string | undefined;
  for (;;) {
    const page = await retry(`listing ${BUCKET}`, () => s3().list({ maxKeys: 1000, ...(continuationToken ? { continuationToken } : {}) }));
    for (const o of page.contents ?? []) out.set(o.key, Number(o.size));
    if (!page.isTruncated || !page.nextContinuationToken) return out;
    continuationToken = page.nextContinuationToken;
  }
}

async function exists(key: string, bytes: number): Promise<boolean> {
  if (!(await retry(`checking ${key}`, () => s3().exists(key)))) return false;
  return (await retry(`checking ${key}`, () => s3().stat(key))).size === bytes;
}

async function upload(key: string, bytes: Uint8Array, type: string): Promise<void> {
  await retry(`uploading ${key}`, () => s3().write(key, bytes, { type }));
}

async function download(key: string): Promise<Uint8Array> {
  return retry(`reading ${key}`, async () => new Uint8Array(await s3().file(key).arrayBuffer()));
}

// ── The manifest ────────────────────────────────────────────────────────────

export function readManifest(): Entry[] {
  if (!existsSync(MANIFEST)) return [];
  return readFileSync(MANIFEST, "utf8")
    .split("\n")
    .filter((l) => l.trim() !== "")
    .map((l, i) => {
      try {
        return JSON.parse(l) as Entry;
      } catch {
        return fail(`${MANIFEST} line ${i + 1} is not JSON`);
      }
    });
}

/** Width and height from a PNG's header, else ffprobe's width, height and duration. */
function measure(file: string, bytes: Uint8Array): Pick<Entry, "width" | "height" | "duration"> {
  const png = [0x89, 0x50, 0x4e, 0x47];
  if (png.every((b, i) => bytes[i] === b) && bytes.length > 24) {
    const v = new DataView(bytes.buffer, bytes.byteOffset);
    return { width: v.getUint32(16), height: v.getUint32(20) };
  }
  const ext = extname(file).toLowerCase();
  if (!TYPES[ext]?.match(/^(image|video)\//)) return {};
  try {
    const out = execFileSync("ffprobe", ["-v", "error", "-show_entries", "stream=width,height:format=duration", "-of", "json", file], { encoding: "utf8" });
    const j = JSON.parse(out) as { streams?: { width?: number; height?: number }[]; format?: { duration?: string } };
    const s = (j.streams ?? []).find((s) => s.width);
    const d = Number(j.format?.duration);
    return {
      ...(s?.width ? { width: s.width, height: s.height } : {}),
      ...(VIDEO.has(ext) && Number.isFinite(d) && d > 0 ? { duration: Math.round(d * 1000) / 1000 } : {}),
    };
  } catch {
    return {};
  }
}

function head(): string | null {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { cwd: ROOT, encoding: "utf8" }).trim();
  } catch {
    return null;
  }
}

export interface AddDesc {
  /** The file; omitted for a benchmark, whose metrics are stored as a JSON file. */
  file?: string;
  feature: string;
  title: string;
  kind?: Kind;
  commit?: string | null;
  date?: string;
  thread?: string | null;
  machine?: string;
  metrics?: Metric[];
  pair?: string;
  side?: "before" | "after";
  source?: string;
  dryRun?: boolean;
}

/**
 * Uploads a file (unless the bucket already holds its bytes) and appends its
 * entry; an entry of the same file, feature and title is not repeated. Returns
 * the entry and whether it is new.
 */
export async function add(d: AddDesc): Promise<{ entry: Entry; added: boolean; uploaded: boolean }> {
  if (!d.feature || !d.title) fail("add needs --feature and --title");
  const thread = threadOf(d.thread);
  if (!thread)
    fail(
      d.thread
        ? `--thread ${d.thread} is not an Amp thread ID (T-…) or thread URL`
        : "add needs the Amp thread that made the artifact: pass --thread T-… (or its URL); neither AMP_THREAD_ID nor AMP_CURRENT_THREAD_ID is set in this shell",
    );
  for (const p of [d.file, d.source]) if (rejectedPath(p)) throw new Refused(`${p} is under a rejected/ directory: withdrawn work is never archived`);
  let bytes: Uint8Array;
  let ext: string;
  if (d.file) {
    if (!existsSync(d.file)) fail(`${d.file} does not exist`);
    bytes = new Uint8Array(readFileSync(d.file));
    ext = extname(d.file).toLowerCase() || ".bin";
  } else {
    if (!d.metrics?.length) fail("add without a file needs --metric (a benchmark)");
    const doc = { feature: d.feature, title: d.title, machine: d.machine ?? null, metrics: d.metrics };
    bytes = new TextEncoder().encode(`${JSON.stringify(doc, null, 2)}\n`);
    ext = ".json";
  }
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  const key = `sha256/${sha256.slice(0, 2)}/${sha256}${ext}`;
  const type = TYPES[ext] ?? "application/octet-stream";
  const manifest = readManifest();
  const withdrawn = manifest.find((e) => e.excluded && e.sha256 === sha256);
  if (withdrawn) throw new Refused(`${d.file ?? d.title} holds the bytes of excluded ${withdrawn.id} (${withdrawn.reason ?? "no reason given"})`);
  const kind: Kind = d.kind ?? (d.metrics?.length && !d.file ? "benchmark" : VIDEO.has(ext) ? "film" : d.side ? "before-after" : "frame");
  if (!KINDS.includes(kind)) fail(`--kind is one of ${KINDS.join(", ")}`);
  if (d.side && !d.pair) fail("--side needs --pair (the name both sides share)");
  const entry: Entry = {
    id: `${d.feature}/${createHash("sha256").update(`${sha256}\n${d.feature}\n${d.title}`).digest("hex").slice(0, 12)}`,
    feature: d.feature,
    title: d.title,
    kind,
    commit: d.commit === undefined ? head() : d.commit,
    date: d.date ?? new Date().toISOString().slice(0, 10),
    thread,
    key,
    sha256,
    bytes: bytes.length,
    type,
    ...(d.file ? measure(d.file, bytes) : {}),
    ...(d.metrics?.length ? { metrics: d.metrics } : {}),
    ...(d.machine ? { machine: d.machine } : {}),
    ...(d.pair ? { pair: d.pair, side: d.side } : {}),
    ...(d.source ? { source: d.source } : {}),
  };
  if (manifest.some((e) => e.id === entry.id)) return { entry, added: false, uploaded: false };
  if (d.dryRun) return { entry, added: true, uploaded: false };
  let uploaded = false;
  if (!(await exists(key, bytes.length))) {
    await upload(key, bytes, type);
    uploaded = true;
  }
  mkdirSync(join(ROOT, "showcase"), { recursive: true });
  appendFileSync(MANIFEST, `${JSON.stringify(entry)}\n`);
  return { entry, added: true, uploaded };
}

/**
 * Withdraws `what` (an entry id, a file, or a sha256): every matching entry is
 * marked excluded with `reason`; content no entry holds yet gets an excluded
 * entry with no key, built from `feature` and `title`, so it is refused later.
 * The bucket's objects are kept (it is private); nothing excluded is shown.
 */
export function exclude(what: string, reason: string, feature?: string, title?: string): Entry[] {
  if (!reason) fail("exclude needs --reason");
  const entries = readManifest();
  let sha: string | undefined;
  let file: string | undefined;
  if (/^[0-9a-f]{64}$/.test(what)) sha = what;
  else if (!entries.some((e) => e.id === what)) {
    if (!existsSync(what)) fail(`${what} is neither an entry id, a file nor a sha256`);
    file = what;
    sha = createHash("sha256").update(readFileSync(what)).digest("hex");
  }
  const hit = (e: Entry) => (sha ? e.sha256 === sha : e.id === what);
  const changed: Entry[] = [];
  const out = entries.map((e) => {
    if (!hit(e) || e.excluded) return e;
    const x: Entry = { ...e, excluded: true, reason };
    changed.push(x);
    return x;
  });
  if (sha && !entries.some(hit)) {
    if (!feature || !title) fail("excluding content no entry holds needs --feature and --title for its record");
    const bytes = file ? readFileSync(file) : undefined;
    const ext = file ? extname(file).toLowerCase() || ".bin" : "";
    const x: Entry = {
      id: `${feature}/${createHash("sha256").update(`${sha}\n${feature}\n${title}`).digest("hex").slice(0, 12)}`,
      feature,
      title,
      kind: "concept",
      commit: null,
      date: new Date().toISOString().slice(0, 10),
      thread: threadOf() ?? null,
      key: null,
      sha256: sha,
      bytes: bytes?.length ?? 0,
      type: TYPES[ext] ?? "application/octet-stream",
      ...(file && bytes ? measure(file, new Uint8Array(bytes)) : {}),
      excluded: true,
      reason,
    };
    out.push(x);
    changed.push(x);
  }
  writeFileSync(MANIFEST, out.map((e) => `${JSON.stringify(e)}\n`).join(""));
  return changed;
}

/** The entries a showcase may show: everything not excluded. */
export const shown = (entries: Entry[]) => entries.filter((e) => !e.excluded);

// ── The index ───────────────────────────────────────────────────────────────

const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Roadmap rows first in the roadmap's order (U, G1, G2, …), then the rest by name. */
function featureOrder(a: string, b: string): number {
  const rank = (f: string) => {
    const m = /^([A-Z]+)(\d+)?(?:-(\d+))?([a-z]*)$/.exec(f);
    return m ? [0, m[1]!, Number(m[2] ?? 0), Number(m[3] ?? 0), m[4]!] : [1, f, 0, 0, ""];
  };
  const x = rank(a);
  const y = rank(b);
  for (let i = 0; i < x.length; i++) if (x[i] !== y[i]) return x[i]! < y[i]! ? -1 : 1;
  return 0;
}

function media(e: Entry, base: string): string {
  const src = `${base}${e.key ?? ""}`;
  const size = e.width ? ` width="${e.width}" height="${e.height}"` : "";
  if (e.type.startsWith("video/")) return `<video src="${src}" controls muted loop preload="metadata"${size}></video>`;
  if (e.type.startsWith("image/")) return `<a href="${src}"><img src="${src}" alt="${esc(e.title)}" loading="lazy"${size}></a>`;
  return `<a class="file" href="${src}">${esc(e.type)} · ${(e.bytes / 1024).toFixed(1)} KB</a>`;
}

function meta(e: Entry): string {
  const bits = [e.date, e.commit ? `<code>${e.commit.slice(0, 8)}</code>` : "", e.thread ? `<a href="https://ampcode.com/threads/${e.thread}">thread</a>` : ""];
  if (e.width) bits.push(`${e.width}×${e.height}`);
  if (e.duration) bits.push(`${e.duration} s`);
  return bits.filter(Boolean).join(" · ");
}

export function indexHtml(all: Entry[], base = "o/"): string {
  const entries = shown(all);
  const features = [...new Set(entries.map((e) => e.feature))].sort(featureOrder);
  const sections = features.map((f) => {
    const all = entries.filter((e) => e.feature === f);
    const pairs = new Map<string, Entry[]>();
    for (const e of all) if (e.pair) pairs.set(e.pair, [...(pairs.get(e.pair) ?? []), e]);
    const cards: string[] = [];
    for (const [name, sides] of pairs) {
      const order = (s: Entry) => (s.side === "before" ? 0 : 1);
      const cells = [...sides].sort((a, b) => order(a) - order(b)).map((s) => `<figure>${media(s, base)}<figcaption><b>${esc(s.side ?? "")}</b> ${esc(s.title)}<small>${meta(s)}</small></figcaption></figure>`);
      cards.push(`<div class="pair"><h3>${esc(name)}</h3><div class="sides">${cells.join("")}</div></div>`);
    }
    const singles = all.filter((e) => !e.pair && e.kind !== "benchmark");
    for (const e of singles) cards.push(`<figure class="${e.kind}">${media(e, base)}<figcaption><span class="kind">${e.kind}</span> ${esc(e.title)}<small>${meta(e)}</small></figcaption></figure>`);
    const bench = all.filter((e) => e.kind === "benchmark");
    const table = bench.length
      ? `<table><thead><tr><th>Benchmark</th><th>Metrics</th><th>Machine</th><th>Commit</th><th>Date</th></tr></thead><tbody>${bench
          .map((e) => `<tr><td>${esc(e.title)}</td><td>${(e.metrics ?? []).map((m) => `${esc(m.name)} <b>${m.value}</b> ${esc(m.unit)}`).join("<br>")}</td><td>${esc(e.machine ?? "")}</td><td>${e.commit ? `<code>${e.commit.slice(0, 8)}</code>` : ""}</td><td>${e.date}</td></tr>`)
          .join("")}</tbody></table>`
      : "";
    const counts = KINDS.map((k) => [k, all.filter((e) => e.kind === k).length] as const).filter(([, n]) => n);
    return `<section id="${esc(f)}"><h2>${esc(f)} <small>${counts.map(([k, n]) => `${n} ${k}`).join(" · ")}</small></h2><div class="grid">${cards.join("")}</div>${table}</section>`;
  });
  const nav = features.map((f) => `<a href="#${esc(f)}">${esc(f)}</a>`).join("");
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Origin Engine showcase archive</title>
<style>
:root{color-scheme:dark;--bg:#0e0f11;--panel:#17191c;--line:#24272c;--text:#e8e9eb;--dim:#8b9098;--accent:#9cc3ff}
*{box-sizing:border-box}body{margin:0;font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif;background:var(--bg);color:var(--text)}
header{position:sticky;top:0;z-index:2;background:rgba(14,15,17,.92);backdrop-filter:blur(8px);border-bottom:1px solid var(--line);padding:14px 24px}
header h1{margin:0 0 8px;font-size:18px;font-weight:600}header p{margin:0 0 8px;color:var(--dim)}
nav{display:flex;flex-wrap:wrap;gap:6px}nav a{color:var(--text);text-decoration:none;background:var(--panel);padding:3px 9px;border-radius:999px;font-size:12px}
main{padding:8px 24px 48px}section{margin-top:28px}h2{font-size:20px;margin:0 0 12px}h2 small{color:var(--dim);font-size:13px;font-weight:400;margin-left:8px}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:14px}
figure{margin:0;background:var(--panel);border-radius:10px;overflow:hidden}figure img,figure video{display:block;width:100%;height:auto;max-height:420px;object-fit:contain;background:#000}
figcaption{padding:8px 10px;font-size:13px}figcaption small{display:block;color:var(--dim);font-size:11px;margin-top:2px}figcaption small a{color:var(--dim)}
.kind{font-size:10px;text-transform:uppercase;letter-spacing:.06em;color:var(--accent);margin-right:4px}
.pair{grid-column:1/-1;background:var(--panel);border-radius:10px;padding:10px}.pair h3{margin:0 0 8px;font-size:14px;font-weight:600}
.sides{display:grid;grid-template-columns:1fr 1fr;gap:10px}.sides figure{background:var(--bg)}
.file{display:block;padding:28px 12px;text-align:center;color:var(--accent)}
table{width:100%;border-collapse:collapse;margin-top:14px;background:var(--panel);border-radius:10px;overflow:hidden;font-size:13px}
th,td{padding:7px 10px;text-align:left;border-bottom:1px solid var(--line);vertical-align:top}th{color:var(--dim);font-weight:500}code{font-size:12px}
</style></head><body>
<header><h1>Origin Engine showcase archive</h1><p>${entries.length} artifacts in ${features.length} features, from showcase/manifest.jsonl (built by bun scripts/showcase.ts index).</p><nav>${nav}</nav></header>
<main>${sections.join("\n")}</main>
</body></html>
`;
}

// ── The command line ────────────────────────────────────────────────────────

function parseArgs(argv: string[]): { positional: string[]; flags: Map<string, string[]> } {
  const positional: string[] = [];
  const flags = new Map<string, string[]>();
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    if (!a.startsWith("--")) {
      positional.push(a);
      continue;
    }
    const name = a.slice(2);
    if (name === "json" || name === "dry-run") {
      flags.set(name, ["true"]);
      continue;
    }
    // --metric takes two words: name=value and its unit.
    const take = name === "metric" ? 2 : 1;
    const vals = argv.slice(i + 1, i + 1 + take);
    if (vals.length < take) fail(`--${name} needs ${take === 2 ? "name=value and a unit" : "a value"}`);
    flags.set(name, [...(flags.get(name) ?? []), vals.join(" ")]);
    i += take;
  }
  return { positional, flags };
}

function parseMetric(s: string): Metric {
  const m = /^([^=\s]+)=(\S+) (\S+)$/.exec(s);
  if (!m) fail(`--metric is name=value unit, such as --metric frame=8.2 ms or --metric hash=0597aacb3ad405dd hex (got '${s}')`);
  const n = Number(m[2]);
  return { name: m[1]!, value: Number.isFinite(n) ? n : m[2]!, unit: m[3]! };
}

async function main(): Promise<void> {
  try {
    await run();
  } catch (e) {
    if (e instanceof Refused) fail(`refused: ${e.message}`);
    throw e;
  }
}

async function run(): Promise<void> {
  const [cmd, ...rest] = process.argv.slice(2);
  const { positional, flags } = parseArgs(rest);
  const one = (n: string) => flags.get(n)?.at(-1);
  switch (cmd) {
    case "add": {
      const side = one("side");
      if (side && side !== "before" && side !== "after") fail("--side is before or after");
      const r = await add({
        file: positional[0],
        feature: one("feature") ?? "",
        title: one("title") ?? "",
        kind: one("kind") as Kind | undefined,
        commit: one("commit"),
        date: one("date"),
        thread: one("thread"),
        machine: one("machine"),
        metrics: (flags.get("metric") ?? []).map(parseMetric),
        pair: one("pair"),
        side: side as "before" | "after" | undefined,
        source: one("source") ?? (positional[0] ? basename(positional[0]) : undefined),
        dryRun: flags.has("dry-run"),
      });
      process.stdout.write(`${r.added ? "added" : "already in the manifest"} ${r.entry.id} ${r.entry.key}${flags.has("dry-run") ? " (dry run: nothing written)" : r.uploaded ? " (uploaded)" : r.added ? " (bytes already in the bucket)" : ""}\n`);
      return;
    }
    case "exclude": {
      if (!positional[0]) fail("usage: exclude <id|file|sha256> --reason \"…\" [--feature F --title T]");
      const changed = exclude(positional[0], one("reason") ?? "", one("feature"), one("title"));
      for (const e of changed) process.stdout.write(`excluded ${e.id} ${e.key ?? "(never uploaded)"}: ${e.reason}\n`);
      if (!changed.length) process.stdout.write("already excluded\n");
      return;
    }
    case "list": {
      const entries = readManifest().filter((e) => (!one("feature") || e.feature === one("feature")) && (!one("kind") || e.kind === one("kind")));
      if (flags.has("json")) process.stdout.write(`${JSON.stringify(entries, null, 2)}\n`);
      else for (const e of entries) process.stdout.write(`${e.id}\t${e.kind}\t${e.date}\t${e.title}${e.excluded ? `\tEXCLUDED: ${e.reason}` : ""}\n`);
      const bytes = new Map(shown(entries).map((e) => [e.key, e.bytes]));
      process.stdout.write(`${entries.length} entries (${entries.length - shown(entries).length} excluded), ${bytes.size} files, ${([...bytes.values()].reduce((a, b) => a + b, 0) / 2 ** 20).toFixed(1)} MiB\n`);
      return;
    }
    case "verify": {
      const entries = readManifest();
      const held = await objects();
      const ids = new Set<string>();
      const problems: string[] = [];
      for (const e of entries) {
        if (ids.has(e.id)) problems.push(`${e.id}: repeated`);
        ids.add(e.id);
        if (!KINDS.includes(e.kind)) problems.push(`${e.id}: kind '${e.kind}'`);
        if (e.excluded && !e.reason) problems.push(`${e.id}: excluded without a reason`);
        if (rejectedPath(e.source) && !e.excluded) problems.push(`${e.id}: ${e.source} is under rejected/`);
        if (e.key === null) {
          if (!e.excluded) problems.push(`${e.id}: no key`);
          continue;
        }
        if (!e.key.includes(e.sha256)) problems.push(`${e.id}: key ${e.key} is not its sha256`);
        const size = held.get(e.key);
        if (size === undefined) problems.push(`${e.id}: ${e.key} is not in ${BUCKET}`);
        else if (size !== e.bytes) problems.push(`${e.id}: ${e.key} is ${size} bytes in ${BUCKET}, ${e.bytes} in the manifest`);
      }
      const keys = new Set(entries.flatMap((e) => (e.key ? [e.key] : [])));
      const total = [...held.values()].reduce((a, b) => a + b, 0);
      process.stdout.write(`${entries.length} entries (${entries.length - shown(entries).length} excluded), ${keys.size} keys; ${BUCKET} holds ${held.size} objects, ${(total / 2 ** 20).toFixed(1)} MiB\n`);
      for (const p of problems) process.stdout.write(`FAIL ${p}\n`);
      if (problems.length) process.exit(1);
      process.stdout.write("every key exists at its size\n");
      return;
    }
    case "index": {
      writeFileSync(INDEX, indexHtml(shown(readManifest())));
      process.stdout.write(`wrote ${INDEX}\n`);
      return;
    }
    case "serve": {
      const port = Number(one("port") ?? 8780);
      mkdirSync(CACHE, { recursive: true });
      const s = Bun.serve({
        port,
        async fetch(req) {
          const path = new URL(req.url).pathname;
          if (path === "/" || path === "/index.html") return new Response(indexHtml(shown(readManifest())), { headers: { "Content-Type": "text/html; charset=utf-8" } });
          if (path.startsWith("/o/sha256/")) {
            const key = path.slice(3);
            const e = shown(readManifest()).find((e) => e.key === key);
            if (!e) return new Response("not in the manifest", { status: 404 });
            const local = join(CACHE, e.sha256);
            if (!existsSync(local)) writeFileSync(local, await download(key));
            return new Response(readFileSync(local), { headers: { "Content-Type": e.type, "Cache-Control": "max-age=31536000, immutable" } });
          }
          return new Response("not found", { status: 404 });
        },
      });
      process.stdout.write(`the showcase index on http://localhost:${s.port}/ (files from ${BUCKET}, cached in ${CACHE})\n`);
      return;
    }
    default:
      fail("usage: bun scripts/showcase.ts add <file> --feature F --title T [options] | exclude <id|file|sha256> --reason R | list | verify | index | serve");
  }
}

if (import.meta.main) await main();
