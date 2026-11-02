// og.motion's weights in the bucket (docs/design.md §2.7): the file
// motion.json pins lives in the private S3 bucket origin-engine-showcase
// (OVHcloud, docs/ci.md §Showcase) under motion/<sha256>.safetensors, so
// pack.sh can fetch it on any machine without a copy of og-motion-train.
//
//   bun scripts/motion-model.ts fetch <out>    download the pinned file to <out>, checked against its sha
//   bun scripts/motion-model.ts upload <file>  put <file> in the bucket, refused unless it is the pinned file
//
// Credentials: OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY (the
// showcase bucket's S3 user); OG_SHOWCASE_BUCKET, OG_SHOWCASE_S3_ENDPOINT and
// OG_SHOWCASE_S3_REGION override the bucket, endpoint and region.
import { S3Client } from "bun";
import { createHash } from "node:crypto";
import { readFileSync, renameSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const pin = JSON.parse(readFileSync(join(import.meta.dir, "..", "motion.json"), "utf8")) as { sha256: string; bytes: number; key: string };

function fail(message: string): never {
  console.error(`motion-model: ${message}`);
  process.exit(1);
}

function client(): S3Client {
  const accessKeyId = process.env.OG_SHOWCASE_S3_ACCESS_KEY;
  const secretAccessKey = process.env.OG_SHOWCASE_S3_SECRET_KEY;
  if (!accessKeyId || !secretAccessKey) fail(`set OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY to fetch ${pin.key} (docs/ci.md §Showcase), or point OG_MOTION_MODEL at the file`);
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

const [command, path] = process.argv.slice(2);
if (command === "fetch" && path) {
  let bytes: Uint8Array;
  try {
    bytes = new Uint8Array(await client().file(pin.key).arrayBuffer());
  } catch (e) {
    fail(`could not read ${pin.key} from the bucket: ${(e as { code?: string }).code ?? e}`);
  }
  check(bytes, pin.key);
  writeFileSync(`${path}.part`, bytes);
  renameSync(`${path}.part`, path);
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
  fail("usage: bun scripts/motion-model.ts fetch <out> | upload <file>");
}
