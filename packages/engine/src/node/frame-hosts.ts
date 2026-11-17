// Remote-frame publish hosts are optional engine data. The signed engine
// package pins their archive hashes; builds fetch them into a shared cache.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { MAX_ARCHIVE, packArchive, packFiles, unpackArchive } from "./vfx-pack.ts";

declare module "node:os" { export function homedir(): string; }

export const FRAME_HOSTS = ["web-publish-frames", "web-publish-2d-frames"] as const;
type Host = typeof FRAME_HOSTS[number];
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const PUBLIC = "https://pub-6dc5404e2ae14e8497e4cee219da67b6.r2.dev/engine/frames";

/** Pack both profiles (single-threaded and lanes), leaving sources intact. */
export function packFrameHosts(dist: string, out: string, base = PUBLIC): void {
  mkdirSync(out, { recursive: true });
  const hosts = Object.fromEntries(FRAME_HOSTS.map((name) => {
    const dir = join(dist, name);
    const bytes = packArchive(dir, name);
    if (bytes.length > MAX_ARCHIVE) throw new Error(`${name}: frame archive exceeds ${MAX_ARCHIVE} bytes`);
    const hash = sha256(bytes);
    writeFileSync(join(out, `${hash}.tar.gz`), bytes);
    const files = Object.fromEntries(packFiles(dir).map((file) => [file, sha256(readFileSync(join(dir, file)))]));
    return [name, { url: `${base}/${hash}.tar.gz`, sha256: hash, bytes: bytes.length, files }];
  }));
  writeFileSync(join(dist, "frame-hosts.json"), `${JSON.stringify(hosts, null, 2)}\n`);
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** A checkout may use its built hosts; an install resolves the signed pin. */
export async function frameHost(engine: string, name: Host): Promise<string> {
  const local = join(engine, "dist", name);
  const pinFile = join(engine, "dist/frame-hosts.json");
  if (!existsSync(pinFile) && existsSync(join(local, "origin-engine_bg.wasm"))) return local;
  const pins: unknown = JSON.parse(readFileSync(pinFile, "utf8"));
  const pin = record(pins) ? pins[name] : undefined;
  if (!record(pin) || typeof pin.url !== "string" || typeof pin.sha256 !== "string"
    || !/^[a-f0-9]{64}$/.test(pin.sha256) || typeof pin.bytes !== "number"
    || !Number.isSafeInteger(pin.bytes) || pin.bytes <= 0 || pin.bytes > MAX_ARCHIVE || !record(pin.files)) {
    throw new Error(`${pinFile}: invalid ${name} pin; reinstall the engine`);
  }
  const files: Array<[string, string]> = [];
  for (const [file, hash] of Object.entries(pin.files)) {
    if (!file.split("/").every((p) => p !== "" && p !== "." && p !== "..") || /[\\:]/.test(file)
      || typeof hash !== "string" || !/^[a-f0-9]{64}$/.test(hash)) throw new Error(`${pinFile}: invalid frame file`);
    files.push([file, hash]);
  }
  for (const required of ["origin-engine.js", "origin-engine_bg.wasm", "origin-engine-lanes.js", "origin-engine-lanes_bg.wasm"]) {
    if (!files.some(([file]) => file === required)) throw new Error(`${pinFile}: missing ${required} pin`);
  }
  const cache = join(process.env.ORIGIN_ENGINE_CACHE || join(homedir(), ".cache/origin-engine"), "frames", pin.sha256);
  const installed = join(cache, name);
  const valid = () => files.every(([file, hash]) => {
    try { return sha256(readFileSync(join(installed, file))) === hash; } catch { return false; }
  });
  if (valid()) return installed;
  if (existsSync(installed)) throw new Error(`${installed}: corrupt frame cache; remove this directory and run og build again while online`);
  mkdirSync(cache, { recursive: true });
  const temporary = mkdtempSync(join(cache, ".fetch-"));
  try {
    const response = await fetch(pin.url, { signal: AbortSignal.timeout(120_000) });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    if (!response.body) throw new Error("empty response body");
    const reader = response.body.getReader();
    const bytes = new Uint8Array(pin.bytes);
    let length = 0;
    try {
      for (;;) {
        const chunk = await reader.read();
        if (chunk.done) break;
        if (length + chunk.value.length > bytes.length) throw new Error("archive exceeds the signed engine pin's length");
        bytes.set(chunk.value, length);
        length += chunk.value.length;
      }
    } finally { await reader.cancel(); }
    if (length !== pin.bytes || sha256(bytes) !== pin.sha256) throw new Error("archive hash or length differs from the signed engine pin");
    unpackArchive(bytes, name, temporary, files.map(([file]) => file));
    for (const [file, hash] of files) {
      if (sha256(readFileSync(join(temporary, name, file))) !== hash) throw new Error(`${file}: checksum mismatch`);
    }
    // Independent build processes may fetch together. Never remove a valid
    // directory another build is already reading.
    if (!valid()) {
      try { renameSync(join(temporary, name), installed); } catch (error) { if (!valid()) throw error; }
    }
    return installed;
  } catch (error) {
    throw new Error(`${name} is unavailable in the verified cache (offline or download refused): ${String(error)}; connect and run og build again`);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}
