// Uploads one version's download-channel files to the platform
// (docs/release.md §3; the owner, 2026-11-05: no GitHub release, no npm),
// which serves them at https://origingame.ai/downloads/engine/<version>/.
// scripts/release.sh publish runs it after its own checks.
//
//   bun scripts/release-publish.ts <dir> --version <v> [--dry-run]
//
// The platform's contract (platform b963d8e), with B the version's URL
// (OG_ENGINE_UPLOAD_URL/<v>, default https://origingame.ai/downloads/engine/<v>)
// and every request carrying `Authorization: Bearer
// $OG_PLATFORM_ENGINE_UPLOAD_TOKEN` (a project secret):
//
//   1. PUT B/SHA256SUMS            the signed list (the archives, manifest.json, and a release's
//                                  install.sh and install.ps1); re-uploading it voids an earlier signature
//   2. PUT B/SHA256SUMS.minisig    verified at once against the engine's key (422 signature_invalid)
//   3. PUT B/<file>                each file the list names (422 sha256_mismatch, not_signed)
//   4. POST B/publish              everything verified again, the manifest checked against the path and
//                                  the list (409 incomplete); answers {version, files, latest, url}
//
// A published version is never overwritten (409 version_exists): a rerun on
// one whose SHA256SUMS on the site is this one reports it published and
// stops; any other bytes are refused. latest/ and the install scripts move
// only with a newer release, never with a dev build.

import { readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

const argv = process.argv.slice(2);
const opt = (n: string) => {
  const i = argv.indexOf(n);
  return i >= 0 ? argv[i + 1] : undefined;
};
const dir = resolve(argv.find((a, i) => !a.startsWith("--") && argv[i - 1] !== "--version") ?? ".");
const version = opt("--version");
const dry = argv.includes("--dry-run");

function die(why: string): never {
  process.stderr.write(`release-publish: ${why}\n`);
  process.exit(1);
}
const say = (s: string) => process.stdout.write(`release-publish: ${s}\n`);
if (!version || !/^\d+\.\d+\.\d+(-dev\.\d+)?$/.test(version)) die("usage: bun scripts/release-publish.ts <dir> --version <v> [--dry-run]");
const base = `${(process.env.OG_ENGINE_UPLOAD_URL ?? "https://origingame.ai/downloads/engine").replace(/\/+$/, "")}/${version}`;
const token = process.env.OG_PLATFORM_ENGINE_UPLOAD_TOKEN;
if (!dry && !token) die("no OG_PLATFORM_ENGINE_UPLOAD_TOKEN (the project secret of origingame/origin-engine)");

const sums = readFileSync(join(dir, "SHA256SUMS"));
const listed = sums.toString("utf8").split("\n").filter(Boolean).map((l) => l.split(/\s+/)[1]!);
if (!listed.includes("manifest.json")) die("SHA256SUMS names no manifest.json: run scripts/release.sh sign (or archive) again");

async function call(method: "PUT" | "POST", path: string, body?: Uint8Array): Promise<{ status: number; text: string }> {
  if (dry) {
    say(`+ ${method} ${base}/${path}${body ? `  ${body.length} bytes` : ""} (dry run)`);
    return { status: 200, text: "" };
  }
  const r = await fetch(`${base}/${path}`, {
    method,
    headers: { authorization: `Bearer ${token}`, ...(body ? { "content-type": "application/octet-stream" } : {}) },
    body,
  });
  return { status: r.status, text: (await r.text()).trim() };
}

// Published already? The site's SHA256SUMS is public: the same bytes are this version, published.
if (!dry) {
  const live = await fetch(`${base}/SHA256SUMS`);
  if (live.ok) {
    const there = new Uint8Array(await live.arrayBuffer());
    if (Buffer.from(there).equals(sums)) {
      say(`${version} is already published with these files: ${base}/`);
      process.exit(0);
    }
    die(`${version} is already published with another SHA256SUMS; a published version never changes`);
  }
}

const expect = (r: { status: number; text: string }, what: string) => {
  if (r.status >= 200 && r.status < 300) return;
  die(`${what} answered ${r.status}: ${r.text.slice(0, 500)}`);
};
expect(await call("PUT", "SHA256SUMS", sums), "PUT SHA256SUMS");
expect(await call("PUT", "SHA256SUMS.minisig", readFileSync(join(dir, "SHA256SUMS.minisig"))), "PUT SHA256SUMS.minisig");
for (const f of listed) {
  const bytes = readFileSync(join(dir, f));
  expect(await call("PUT", f, bytes), `PUT ${f}`);
  if (!dry) say(`${f} uploaded (${statSync(join(dir, f)).size} bytes)`);
}
const done = await call("POST", "publish");
expect(done, "POST publish");
say(dry ? "dry run; nothing uploaded" : `published ${version}: ${done.text}`);
