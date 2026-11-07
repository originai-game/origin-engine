// A downloads tree laid out like https://origingame.ai/downloads/engine for
// og vfx add's tests: <downloads>/<version>/ with vfx/spark-pack.tar.gz,
// manifest.json and SHA256SUMS written by scripts/release-dist.ts's own
// writeVfxArchives and finish, then SHA256SUMS signed (minisign's format)
// with a fresh test key. Prints `pubkey <key line>` last.
//
//   bun tests/fixtures/vfx-release/channel.ts <downloads> <version> [--evil traversal|symlink]
//
// --evil rewrites one entry of the archive (a `..` path, or a symlink) and
// signs the result, so the archive is the only thing wrong with the release.

import { createHash, generateKeyPairSync, randomBytes, sign } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { gunzipSync, gzipSync } from "node:zlib";
import { finish, writeVfxArchives } from "../../../scripts/release-dist.ts";

const [out, version, flag, evil] = process.argv.slice(2);
if (!out || !version || (flag && (flag !== "--evil" || !["traversal", "symlink"].includes(evil ?? "")))) {
  process.stderr.write("usage: bun tests/fixtures/vfx-release/channel.ts <downloads> <version> [--evil traversal|symlink]\n");
  process.exit(2);
}
// Never the release key, whatever the environment holds.
delete process.env.OG_ENGINE_MINISIGN_KEY;
const dir = join(resolve(out), version);
mkdirSync(dir, { recursive: true });
const vfx = writeVfxArchives(dir, version, import.meta.dir);

if (evil) {
  const file = join(dir, "vfx/spark-pack.tar.gz");
  const tar = gunzipSync(readFileSync(file));
  const victim = "spark-pack/glint.ogt";
  let at = 0;
  while (tar.subarray(at, at + victim.length + 1).toString("latin1") !== `${victim}\0`) at += 512;
  const h = tar.subarray(at, at + 512);
  if (evil === "traversal") h.write("spark-pack/../glint.ogt\0", 0, "latin1");
  else {
    h[156] = 0x32;
    h.write("/etc/passwd\0", 157, "latin1");
  }
  h.fill(0x20, 148, 156);
  h.write(`${h.reduce((a, b) => a + b, 0).toString(8).padStart(6, "0")}\0 `, 148, "latin1");
  const bytes = gzipSync(tar);
  writeFileSync(file, bytes);
  vfx["spark-pack"] = { ...vfx["spark-pack"]!, sha256: createHash("sha256").update(bytes).digest("hex"), size: bytes.length };
}
finish(dir, version, "0".repeat(40), "2026-11-07T00:00:00.000Z", vfx);

const { publicKey, privateKey } = generateKeyPairSync("ed25519");
const id = randomBytes(8);
const pubkey = Buffer.concat([Buffer.from("Ed"), id, publicKey.export({ format: "der", type: "spki" }).subarray(12)]).toString("base64");
const comment = `origin-engine ${version} test`;
const sig = sign(null, createHash("blake2b512").update(readFileSync(join(dir, "SHA256SUMS"))).digest(), privateKey);
const global = sign(null, Buffer.concat([sig, Buffer.from(comment)]), privateKey);
writeFileSync(join(dir, "SHA256SUMS.minisig"), `untrusted comment: test key\n${Buffer.concat([Buffer.from("ED"), id, sig]).toString("base64")}\ntrusted comment: ${comment}\n${global.toString("base64")}\n`);
process.stdout.write(`pubkey ${pubkey}\n`);
