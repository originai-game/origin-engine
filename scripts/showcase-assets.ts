// A showcase project's binary assets (models, terrain maps, textures) are
// not in git: its `assets.json` pins each one (`file`, `sha256`, `bytes`,
// `key`, `url`), as packages/engine/motion.json pins og.motion's weights
// (docs/ci.md §The model bucket). Every machine reads them anonymously from
// `url`, the public R2 bucket origin-engine-models; the private S3 bucket
// origin-engine-showcase (OVHcloud) under the same key,
// showcase/<project>/<sha256>.<ext>, is the fallback and the upload target.
//
//   bun scripts/showcase-assets.ts ensure <project>           install every pin, reusing verified local files; run it before the game is cooked
//   bun scripts/showcase-assets.ts verify <project>           read and verify every remote pin, even with local copies
//   bun scripts/showcase-assets.ts pin <project> <file>...    pin the project's files as they are now (paths within the project), keeping the other pins
//   bun scripts/showcase-assets.ts upload <project>           put every pinned file in both buckets, refused for a file that is not its pin
//
// <project> is the project's directory (showcase/first-light); its last
// segment names it in the buckets. Credentials, for upload and the fallback
// only: OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY (the showcase
// bucket's S3 user; OG_SHOWCASE_BUCKET, OG_SHOWCASE_S3_ENDPOINT and
// OG_SHOWCASE_S3_REGION override the bucket, endpoint and region), and for
// R2 CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN (the token wrangler
// takes). OG_SHOWCASE_PUBLIC_URL and OG_SHOWCASE_R2_API override where the
// public bucket is read and written.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { basename, dirname, extname, isAbsolute, join, normalize, resolve } from "node:path";

interface Pin {
  file: string;
  bytes: number;
  sha256: string;
  key: string;
  url: string;
}

const PUBLIC_URL = process.env.OG_SHOWCASE_PUBLIC_URL ?? "https://pub-6dc5404e2ae14e8497e4cee219da67b6.r2.dev";
const R2_BUCKET = "origin-engine-models";
/** Pins read or written at once. */
const PARALLEL = 6;

function fail(message: string): never {
  console.error(`showcase-assets: ${message}`);
  process.exit(1);
}

/** The part of Bun's S3 client this script uses. */
interface S3 {
  file(key: string): { arrayBuffer(): Promise<ArrayBuffer> };
  exists(key: string): Promise<boolean>;
  stat(key: string): Promise<{ size: number }>;
  write(key: string, data: Uint8Array, options: { type: string }): Promise<number>;
}
type S3Options = { accessKeyId: string; secretAccessKey: string; endpoint: string; region: string; bucket: string };
const { S3Client } = (globalThis as unknown as { Bun: { S3Client: new (options: S3Options) => S3 } }).Bun;

function bucket(why: string): S3 {
  const accessKeyId = process.env.OG_SHOWCASE_S3_ACCESS_KEY;
  const secretAccessKey = process.env.OG_SHOWCASE_S3_SECRET_KEY;
  if (!accessKeyId || !secretAccessKey) fail(`set OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY to ${why} (docs/ci.md §The model bucket)`);
  return new S3Client({
    accessKeyId,
    secretAccessKey,
    endpoint: process.env.OG_SHOWCASE_S3_ENDPOINT ?? "https://s3.bhs.io.cloud.ovh.net",
    region: process.env.OG_SHOWCASE_S3_REGION ?? "bhs",
    bucket: process.env.OG_SHOWCASE_BUCKET ?? "origin-engine-showcase",
  });
}

const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** What is wrong with `bytes` as `pin`, or nothing. */
function mismatch(bytes: Uint8Array, pin: Pin, what: string): string | undefined {
  const sha = sha256(bytes);
  if (bytes.length === pin.bytes && sha === pin.sha256) return undefined;
  return `${what} is ${bytes.length} bytes, sha256 ${sha}; assets.json pins ${pin.file} at ${pin.bytes} bytes, sha256 ${pin.sha256}`;
}

class Project {
  readonly root: string;
  readonly name: string;
  readonly manifest: string;
  pins: Pin[] = [];

  constructor(directory: string | undefined, mustExist: boolean) {
    if (!directory) fail("name the project's directory, such as showcase/first-light");
    this.root = resolve(directory);
    this.name = basename(this.root);
    this.manifest = join(this.root, "assets.json");
    if (!existsSync(join(this.root, "project.ogt"))) fail(`${directory} is not a project: it has no project.ogt`);
    if (!existsSync(this.manifest)) {
      if (mustExist) fail(`${this.manifest} is missing: the project pins no assets (pin them with \`pin\`)`);
      return;
    }
    const read = JSON.parse(readFileSync(this.manifest, "utf8")) as { files?: Pin[] };
    if (!Array.isArray(read.files)) fail(`${this.manifest}: expected { "files": [...] }`);
    for (const pin of read.files) {
      if (typeof pin.file !== "string" || typeof pin.sha256 !== "string" || typeof pin.bytes !== "number" || typeof pin.key !== "string" || typeof pin.url !== "string") {
        fail(`${this.manifest}: a pin needs file, sha256, bytes, key and url: ${JSON.stringify(pin)}`);
      }
      this.inside(pin.file);
    }
    this.pins = read.files;
  }

  /** A pinned file's path, refused when it leaves the project. */
  inside(file: string): string {
    const path = normalize(file);
    if (isAbsolute(path) || path.startsWith("..")) fail(`${this.manifest}: ${file} is not a path within the project`);
    return join(this.root, path);
  }

  write(): void {
    this.pins.sort((a, b) => (a.file < b.file ? -1 : a.file > b.file ? 1 : 0));
    writeFileSync(this.manifest, `${JSON.stringify({ files: this.pins }, null, 2)}\n`);
  }
}

/** The pinned bytes from the public bucket, or from the private one when that fails. */
async function remote(pin: Pin): Promise<Uint8Array> {
  try {
    const response = await fetch(pin.url);
    if (response.ok) {
      const bytes = new Uint8Array(await response.arrayBuffer());
      const wrong = mismatch(bytes, pin, pin.url);
      if (!wrong) return bytes;
      console.error(`showcase-assets: ${wrong}; trying the bucket`);
    } else {
      console.error(`showcase-assets: ${pin.url} answered ${response.status}; trying the bucket`);
    }
  } catch (e) {
    console.error(`showcase-assets: could not read ${pin.url} (${e}); trying the bucket`);
  }
  let bytes: Uint8Array;
  try {
    bytes = new Uint8Array(await bucket(`fetch ${pin.key}`).file(pin.key).arrayBuffer());
  } catch (e) {
    fail(`could not read ${pin.file} (${pin.key}) from the bucket: ${(e as { code?: string }).code ?? e}`);
  }
  const wrong = mismatch(bytes, pin, pin.key);
  if (wrong) fail(wrong);
  return bytes;
}

function install(path: string, bytes: Uint8Array): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(`${path}.part`, bytes);
  renameSync(`${path}.part`, path);
}

/** Runs `job` over the pins, a few at once. */
async function each(pins: readonly Pin[], job: (pin: Pin) => Promise<void>): Promise<void> {
  let next = 0;
  await Promise.all(Array.from({ length: Math.min(PARALLEL, pins.length) }, async () => {
    while (next < pins.length) await job(pins[next++]!);
  }));
}

/** Puts the bytes in the public R2 bucket, through the API wrangler uses. */
async function publish(pin: Pin, bytes: Uint8Array): Promise<void> {
  const account = process.env.CLOUDFLARE_ACCOUNT_ID, token = process.env.CLOUDFLARE_API_TOKEN;
  if (!account || !token) fail(`set CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN to publish ${pin.key}, or run: wrangler r2 object put ${R2_BUCKET}/${pin.key} --file ${pin.file} --remote`);
  const api = process.env.OG_SHOWCASE_R2_API ?? `https://api.cloudflare.com/client/v4/accounts/${account}/r2/buckets/${R2_BUCKET}/objects`;
  const response = await fetch(`${api}/${pin.key.split("/").map(encodeURIComponent).join("/")}`, {
    method: "PUT",
    headers: { authorization: `Bearer ${token}`, "content-type": "application/octet-stream" },
    body: bytes,
  });
  if (!response.ok) fail(`could not publish ${pin.key} to ${R2_BUCKET}: ${response.status} ${(await response.text()).slice(0, 300)}`);
}

const [command, directory, ...files] = process.argv.slice(2);
if (command === "ensure") {
  const project = new Project(directory, true);
  let fetched = 0;
  await each(project.pins, async (pin) => {
    const path = project.inside(pin.file);
    if (existsSync(path) && !mismatch(new Uint8Array(readFileSync(path)), pin, path)) return;
    install(path, await remote(pin));
    fetched++;
  });
  console.log(`showcase-assets: ${project.name}: ${project.pins.length} pinned files ready (${fetched} fetched)`);
} else if (command === "verify") {
  const project = new Project(directory, true);
  await each(project.pins, async (pin) => {
    await remote(pin);
  });
  console.log(`showcase-assets: ${project.name}: verified ${project.pins.length} remote pins`);
} else if (command === "pin" && files.length > 0) {
  const project = new Project(directory, false);
  for (const file of files) {
    const path = project.inside(file);
    if (!existsSync(path)) fail(`${path} does not exist`);
    const bytes = new Uint8Array(readFileSync(path));
    const sha = sha256(bytes);
    const key = `showcase/${project.name}/${sha}${extname(file)}`;
    const pin: Pin = { file: normalize(file).split("\\").join("/"), bytes: bytes.length, sha256: sha, key, url: `${PUBLIC_URL}/${key}` };
    project.pins = project.pins.filter((p) => p.file !== pin.file);
    project.pins.push(pin);
  }
  project.write();
  console.log(`showcase-assets: ${project.name}: pinned ${files.length} files, ${project.pins.length} in ${project.manifest}; upload them before committing it`);
} else if (command === "upload") {
  const project = new Project(directory, true);
  const s3 = bucket("upload");
  let put = 0;
  await each(project.pins, async (pin) => {
    const path = project.inside(pin.file);
    if (!existsSync(path)) fail(`${path} does not exist: only a machine holding the pinned files uploads them`);
    const bytes = new Uint8Array(readFileSync(path));
    const wrong = mismatch(bytes, pin, path);
    if (wrong) fail(`${wrong}; pin it again or restore it`);
    if (!((await s3.exists(pin.key)) && (await s3.stat(pin.key)).size === pin.bytes)) {
      await s3.write(pin.key, bytes, { type: "application/octet-stream" });
      put++;
    }
    // The public copy is checked as a fetch would read it, and written only when that fails.
    const published = await fetch(pin.url).then(async (r) => r.ok && !mismatch(new Uint8Array(await r.arrayBuffer()), pin, pin.url), () => false);
    if (!published) {
      await publish(pin, bytes);
      put++;
    }
  });
  console.log(`showcase-assets: ${project.name}: ${project.pins.length} pinned files in both buckets (${put} puts)`);
} else {
  fail("usage: bun scripts/showcase-assets.ts ensure <project> | verify <project> | pin <project> <file>... | upload <project>");
}
