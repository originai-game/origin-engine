// og.motion's weights in the bucket (docs/design.md §2.7): the file
// motion.json pins lives in the private S3 bucket origin-engine-showcase
// (OVHcloud, docs/ci.md §The model bucket) under motion/<sha256>.safetensors, so
// pack.sh can fetch it on any machine without a copy of og-motion-train.
//
//   bun scripts/motion-model.ts fetch <out>    download the pinned file to <out>, checked against its sha
//   bun scripts/motion-model.ts upload <file>  put <file> in the bucket, refused unless it is the pinned file
//   bun scripts/motion-model.ts ensure         install the pin in the package, reusing a verified local copy
//   bun scripts/motion-model.ts verify         read and verify the remote pin, even with a local copy
//
// Credentials: OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY (the
// showcase bucket's S3 user); OG_SHOWCASE_BUCKET, OG_SHOWCASE_S3_ENDPOINT and
// OG_SHOWCASE_S3_REGION override the bucket, endpoint and region.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const pkg = fileURLToPath(new URL("..", import.meta.url));
const pin = JSON.parse(readFileSync(join(pkg, "motion.json"), "utf8")) as { file: string; sha256: string; bytes: number; key: string };

function fail(message: string): never {
  console.error(`motion-model: ${message}`);
  process.exit(1);
}

/** The part of Bun's S3 client this script uses (src/env.d.ts declares the Bun global only as far as the runtime reads it). */
interface S3 {
  file(key: string): { arrayBuffer(): Promise<ArrayBuffer> };
  exists(key: string): Promise<boolean>;
  stat(key: string): Promise<{ size: number }>;
  write(key: string, data: Uint8Array, options: { type: string }): Promise<number>;
}
type S3Options = { accessKeyId: string; secretAccessKey: string; endpoint: string; region: string; bucket: string };
const { S3Client } = (globalThis as unknown as { Bun: { S3Client: new (options: S3Options) => S3 } }).Bun;

function client(): S3 {
  const accessKeyId = process.env.OG_SHOWCASE_S3_ACCESS_KEY;
  const secretAccessKey = process.env.OG_SHOWCASE_S3_SECRET_KEY;
  if (!accessKeyId || !secretAccessKey) fail(`set OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY to fetch ${pin.key} (docs/ci.md §The model bucket), or point OG_MOTION_MODEL at the file`);
  return new S3Client({
    accessKeyId,
    secretAccessKey,
    endpoint: process.env.OG_SHOWCASE_S3_ENDPOINT ?? "https://s3.bhs.io.cloud.ovh.net",
    region: process.env.OG_SHOWCASE_S3_REGION ?? "bhs",
    bucket: process.env.OG_SHOWCASE_BUCKET ?? "origin-engine-showcase",
  });
}

function check(bytes: Uint8Array, what: string): void {
  const sha = createHash("sha256").update(bytes).digest("hex");
  if (bytes.length !== pin.bytes || sha !== pin.sha256) fail(`${what} is ${bytes.length} bytes, sha256 ${sha}; motion.json pins ${pin.bytes} bytes, sha256 ${pin.sha256}`);
}

async function remote(): Promise<Uint8Array> {
  let bytes: Uint8Array;
  try {
    bytes = new Uint8Array(await client().file(pin.key).arrayBuffer());
  } catch (e) {
    fail(`could not read ${pin.key} from the bucket: ${(e as { code?: string }).code ?? e}`);
  }
  check(bytes, pin.key);
  return bytes;
}

function install(path: string, bytes: Uint8Array): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(`${path}.part`, bytes);
  renameSync(`${path}.part`, path);
}

const [command, path] = process.argv.slice(2);
if (command === "fetch" && path) {
  install(path, await remote());
} else if (command === "verify") {
  await remote();
  console.log(`motion-model: verified remote ${pin.key} (${pin.bytes} bytes)`);
} else if (command === "ensure") {
  const destination = join(pkg, pin.file);
  const override = process.env.OG_MOTION_MODEL;
  if (override) {
    const bytes = new Uint8Array(readFileSync(override));
    check(bytes, override);
    install(destination, bytes);
  } else {
    const bytes = existsSync(destination) ? new Uint8Array(readFileSync(destination)) : undefined;
    if (!bytes || bytes.length !== pin.bytes || createHash("sha256").update(bytes).digest("hex") !== pin.sha256) {
      install(destination, await remote());
    }
  }
  console.log(`motion-model: pinned weights ready at ${destination}`);
} else if (command === "upload" && path) {
  const bytes = new Uint8Array(readFileSync(path));
  check(bytes, path);
  const s3 = client();
  if ((await s3.exists(pin.key)) && (await s3.stat(pin.key)).size === pin.bytes) {
    console.log(`motion-model: ${pin.key} is already in the bucket`);
  } else {
    await s3.write(pin.key, bytes, { type: "application/octet-stream" });
    console.log(`motion-model: uploaded ${pin.key} (${pin.bytes} bytes)`);
  }
} else {
  fail("usage: bun scripts/motion-model.ts ensure | verify | fetch <out> | upload <file>");
}
