// Private transport for packing intermediates; never a download channel or publish API.
// The platform owns authentication, immutable storage and retention (docs/release.md §3).
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { VERSION_RE } from "../packages/launcher/og.ts";

export const PACK_TARGETS = ["linux-x64", "darwin-arm64", "win32-x64"] as const;
export type PackTarget = typeof PACK_TARGETS[number];
export interface PackRecord {
  schema: 1;
  version: string;
  revision: string;
  target: PackTarget;
  dirty: boolean;
  elapsedSeconds: number;
  files: Record<string, { sha256: string; size: number }>;
}
export const recordName = (target: PackTarget) => `pack-${target}.json`;
export function packFiles(version: string, target: PackTarget): string[] {
  return [`origingame-engine-${target}-${version}.tgz`, ...(target === "linux-x64" ? [
    `origingame-engine-${version}.tgz`, `origingame-create-game-${version}.tgz`,
    `origingame-design-${version}.tgz`, `origingame-engine-wasm-symbols-${version}.tgz`, "NOTES.md",
  ] : [])].sort();
}
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export function checkPack(record: PackRecord, version: string, revision: string, target: PackTarget, dir?: string): void {
  if (record.schema !== 1 || record.version !== version || record.revision !== revision || record.target !== target
    || typeof record.dirty !== "boolean" || !Number.isFinite(record.elapsedSeconds) || record.elapsedSeconds < 0)
    throw new Error(`${target}: pack identity differs from ${version} at ${revision}`);
  const names = packFiles(version, target);
  if (JSON.stringify(Object.keys(record.files).sort()) !== JSON.stringify(names)) throw new Error(`${target}: unexpected pack file set`);
  for (const name of names) {
    const file = record.files[name]!;
    if (!/^[a-f0-9]{64}$/.test(file.sha256) || !Number.isSafeInteger(file.size) || file.size <= 0) throw new Error(`${name}: invalid sha256 or size`);
    if (dir) {
      const bytes = readFileSync(join(dir, name));
      if (bytes.length !== file.size || digest(bytes) !== file.sha256) throw new Error(`${name}: sha256 or size differs from pack record`);
    }
  }
}
/** Used again immediately before archive assembly, not just during transport. */
export function checkPacks(dir: string, version: string, revision: string, targets: PackTarget[]): PackRecord[] {
  return [...new Set<PackTarget>(["linux-x64", ...targets])].map((target) => {
    const record = JSON.parse(readFileSync(join(dir, recordName(target)), "utf8")) as PackRecord;
    checkPack(record, version, revision, target, dir);
    return record;
  });
}

async function main() {
  const [verb, ...args] = process.argv.slice(2);
  const opt = (name: string) => { const i = args.indexOf(name); return i < 0 ? undefined : args[i + 1]; };
  const version = opt("--version");
  const revision = opt("--revision");
  const target = opt("--target") as PackTarget;
  if (!version || !VERSION_RE.test(version) || !revision || !/^[a-f0-9]{40}$/.test(revision))
    throw new Error("stage requires --version <v> --revision <full commit>");
  if (!["record", "push", "pull"].includes(verb ?? "")) throw new Error("stage verb is record, push or pull");
  if (verb !== "pull" && !PACK_TARGETS.includes(target)) throw new Error("stage requires --target linux-x64|darwin-arm64|win32-x64");
  const dir = resolve(opt("--dir") ?? `dist/release/${version}`);
  mkdirSync(dir, { recursive: true });
  if (verb === "record") {
    const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    if (head !== revision) throw new Error("HEAD changed while packing");
    const record: PackRecord = {
      schema: 1, version, revision, target, dirty: args.includes("--dirty"), elapsedSeconds: Number(opt("--elapsed")),
      files: Object.fromEntries(packFiles(version, target).map((name) => {
        const bytes = readFileSync(join(dir, name));
        return [name, { sha256: digest(bytes), size: bytes.length }];
      })),
    };
    checkPack(record, version, revision, target, dir);
    writeFileSync(join(dir, recordName(target)), `${JSON.stringify(record, null, 2)}\n`);
    console.log(`${target}: packed ${version} at ${revision} in ${record.elapsedSeconds}s${record.dirty ? " (dirty; cannot publish)" : ""}`);
    return;
  }
  const base = process.env.OG_ENGINE_STAGING_URL;
  const token = process.env.OG_ENGINE_STAGING_TOKEN;
  if (!base || !token) throw new Error("set OG_ENGINE_STAGING_URL and OG_ENGINE_STAGING_TOKEN (never the publish token on a packing host)");
  const url = new URL(base);
  if (url.protocol !== "https:" && !(url.protocol === "http:" && ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname))) throw new Error("staging requires HTTPS");
  if (url.username || url.password || url.search || url.hash) throw new Error("staging base must not contain credentials, query or fragment");
  const request = async (target: PackTarget, name: string, bytes?: Uint8Array): Promise<Response> => {
    const response = await fetch(`${base.replace(/\/$/, "")}/${version}/${revision}/${target}/${name}`, {
      method: bytes ? "PUT" : "GET", redirect: "error",
      headers: { Authorization: `Bearer ${token}`, ...(bytes ? { "Content-Type": "application/octet-stream", "X-Content-SHA256": digest(bytes) } : {}) },
      body: bytes,
    });
    if (!response.ok) throw new Error(`${target}/${name}: staging ${bytes ? "PUT" : "GET"} returned ${response.status}`);
    return response;
  };
  if (verb === "push") {
    const record = JSON.parse(readFileSync(join(dir, recordName(target)), "utf8")) as PackRecord;
    checkPack(record, version, revision, target, dir);
    for (const name of packFiles(version, target)) await request(target, name, readFileSync(join(dir, name)));
    // A record is the completion marker, uploaded only after every payload succeeded.
    await request(target, "pack.json", readFileSync(join(dir, recordName(target))));
    console.log(`${target}: staged ${Object.keys(record.files).length} files at ${version}/${revision}`);
  } else {
    for (const target of PACK_TARGETS) {
      const record = await (await request(target, "pack.json")).json() as PackRecord;
      checkPack(record, version, revision, target);
      // Never leave a valid old completion marker beside a partially replaced download.
      rmSync(join(dir, recordName(target)), { force: true });
      for (const name of packFiles(version, target)) {
        const bytes = new Uint8Array(await (await request(target, name)).arrayBuffer());
        const file = record.files[name]!;
        if (bytes.length !== file.size || digest(bytes) !== file.sha256) throw new Error(`${name}: staging sha256 or size mismatch`);
        writeFileSync(join(dir, `${name}.partial`), bytes);
        renameSync(join(dir, `${name}.partial`), join(dir, name));
      }
      writeFileSync(join(dir, recordName(target)), `${JSON.stringify(record, null, 2)}\n`);
      console.log(`${target}: pulled and verified ${Object.keys(record.files).length} files from ${revision}`);
    }
  }
}
if (import.meta.main) await main();
