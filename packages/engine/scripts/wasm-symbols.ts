// Shipping keeps the bindgen instructions unchanged. Only the optional name
// section moves to the separate, matching named module for offline symbolication.
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export function withoutNames(wasm: Uint8Array): Uint8Array {
  const bytes = wasm;
  if (bytes.length < 8 || ![0, 97, 115, 109, 1, 0, 0, 0].every((b, i) => bytes[i] === b)) throw new Error("invalid wasm header");
  const parts = [bytes.subarray(0, 8)];
  let at = 8;
  const leb = () => {
    let value = 0;
    for (let shift = 0; shift < 35; shift += 7) {
      const byte = bytes[at++];
      if (byte === undefined) throw new Error("truncated wasm section length");
      value += (byte & 127) * 2 ** shift;
      if (!(byte & 128)) return value;
    }
    throw new Error("invalid wasm section length");
  };
  while (at < bytes.length) {
    const start = at;
    const id = bytes[at++];
    const size = leb();
    const end = at + size;
    if (end > bytes.length) throw new Error("truncated wasm section");
    let name = "";
    if (id === 0) {
      const length = leb();
      if (at + length > end) throw new Error("truncated wasm custom section name");
      name = new TextDecoder().decode(bytes.subarray(at, at + length));
    }
    if (name !== "name") parts.push(bytes.subarray(start, end));
    at = end;
  }
  const result = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) { result.set(part, offset); offset += part.length; }
  return result;
}

/** Each web host directory of dist, and its split modules' directories (`modules*`, scripts/wasm-split.ts). */
function wasmDirs(dist: string): string[] {
  return readdirSync(dist, { withFileTypes: true })
    .filter((d) => d.isDirectory() && d.name.startsWith("web"))
    .flatMap((d) => [d.name, ...readdirSync(join(dist, d.name), { withFileTypes: true }).filter((m) => m.isDirectory() && m.name.startsWith("modules")).map((m) => join(d.name, m.name))]);
}

export function splitSymbols(dist: string, symbols: string): void {
  for (const dir of wasmDirs(dist)) {
    for (const file of readdirSync(join(dist, dir))) {
      if (!file.endsWith(".wasm")) continue;
      const path = join(dist, dir, file);
      const named = readFileSync(path);
      const stripped = withoutNames(named);
      if (stripped.length === named.length) continue;
      mkdirSync(join(symbols, dir), { recursive: true });
      writeFileSync(join(symbols, dir, file), named);
      writeFileSync(path, stripped);
    }
  }
}

if (import.meta.main) {
  const [dist, symbols] = process.argv.slice(2);
  if (!dist || !symbols) throw new Error("usage: bun wasm-symbols.ts <dist> <symbols>");
  splitSymbols(dist, symbols);
}
