// What a build keeps between runs (src/node/web-build.ts: a file's
// compressed size, a host's probe) is kept in the user's cache directory,
// never in the engine's install, and builds side by side keep each other's.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ENGINE_FACTS, factGet, factPut, factsFile } from "../src/node/web-build.ts";
import { buildGfxWasm } from "./support/gfx-build.ts";

declare const Bun: { spawnSync(cmd: string[], o: { env: Record<string, string | undefined> }): { exitCode: number } };

const module = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "node", "web-build.ts");

test("the facts file is the user's, and OG_BUILD_CACHE moves it or turns it off", () => {
  expect(factsFile({ HOME: "/home/a" })).toBe(join("/home/a", ".cache", "origingame", "build-v1", "facts.json"));
  expect(factsFile({ LOCALAPPDATA: "C:\\Users\\a\\AppData\\Local" })).toBe(join("C:\\Users\\a\\AppData\\Local", "origingame", "build-v1", "facts.json"));
  expect(factsFile({ HOME: "/home/a", OG_BUILD_CACHE: "/var/cache/og" })).toBe(join("/var/cache/og", "facts.json"));
  expect(factsFile({ HOME: "/home/a", OG_BUILD_CACHE: "off" })).toBeNull();
  expect(factsFile({ HOME: "/home/a", OG_BUILD_CACHE: "" })).toBeNull();
  expect(factsFile({})).toBeNull();
});

test("a fact is written to the cache directory, read by the next build, and kept beside another build's", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-build-facts-"));
  const before = process.env.OG_BUILD_CACHE;
  const shippedBefore = existsSync(ENGINE_FACTS) ? readFileSync(ENGINE_FACTS, "utf8") : null;
  try {
    process.env.OG_BUILD_CACHE = dir;
    expect(factGet("size:test:first")).toBeUndefined();
    factPut("size:test:first", { size: 1 });
    expect(JSON.parse(readFileSync(join(dir, "facts.json"), "utf8"))).toEqual({ "size:test:first": { size: 1 } });
    // Another build: a process of its own, which reads the first fact and keeps a second.
    const script = `const m = await import(${JSON.stringify(module)}); if (m.factGet("size:test:first")?.size !== 1) process.exit(3); m.factPut("size:test:second", { size: 2 });`;
    expect(Bun.spawnSync(["bun", "-e", script], { env: { ...process.env, OG_BUILD_CACHE: dir } }).exitCode).toBe(0);
    // This build read the file before the other wrote to it; its next fact does not drop the other's.
    factPut("size:test:third", { size: 3 });
    expect(Object.keys(JSON.parse(readFileSync(join(dir, "facts.json"), "utf8")) as object).sort()).toEqual(["size:test:first", "size:test:second", "size:test:third"]);
    expect(factGet("size:test:second")).toEqual({ size: 2 });
    // Nothing was written into the engine's own directory.
    expect(existsSync(ENGINE_FACTS) ? readFileSync(ENGINE_FACTS, "utf8") : null).toBe(shippedBefore);
  } finally {
    if (before === undefined) delete process.env.OG_BUILD_CACHE;
    else process.env.OG_BUILD_CACHE = before;
    rmSync(dir, { recursive: true, force: true });
  }
});

test("with the cache off a build keeps its facts for itself and writes nothing", () => {
  const before = process.env.OG_BUILD_CACHE;
  try {
    process.env.OG_BUILD_CACHE = "off";
    factPut("size:test:unwritten", { size: 4 });
    expect(factGet("size:test:unwritten")).toEqual({ size: 4 });
  } finally {
    if (before === undefined) delete process.env.OG_BUILD_CACHE;
    else process.env.OG_BUILD_CACHE = before;
  }
});

test("gfx builds reject unverified, zeroed and header-preserving corrupted cached wasm", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-gfx-integrity-"));
  const raw = join(dir, "og_tests_web.wasm");
  // Valid modules differing only in a custom section, so magic alone
  // cannot distinguish the compiler's output from the corrupt cache.
  const compiled = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0, 0, 2, 1, 97]);
  const changed = Uint8Array.from(compiled);
  changed[11] = 98;
  let fresh = true;
  let brokenBuild = false;
  const run = (command: string[]) => {
    if (command[1] === "clean") {
      fresh = false;
      return "";
    }
    if (!fresh) writeFileSync(raw, brokenBuild ? new Uint8Array(compiled.length) : compiled);
    const result = JSON.stringify({ reason: "compiler-artifact", target: { name: "og_tests_web" }, filenames: [raw], fresh });
    fresh = true;
    return result;
  };
  try {
    // A fresh artifact from before receipts existed is not silently blessed.
    writeFileSync(raw, changed);
    expect(Array.from(readFileSync(buildGfxWasm(run, dir, "wasm-test")))).toEqual(Array.from(compiled));
    writeFileSync(raw, new Uint8Array(compiled.length));
    expect(Array.from(readFileSync(buildGfxWasm(run, dir, "wasm-test")))).toEqual(Array.from(compiled));
    writeFileSync(raw, changed);
    expect(Array.from(readFileSync(buildGfxWasm(run, dir, "wasm-test")))).toEqual(Array.from(compiled));
    // A compiler that returns a broken NEW output must fail, not attest it.
    brokenBuild = true;
    writeFileSync(raw, new Uint8Array(compiled.length));
    expect(() => buildGfxWasm(run, dir, "wasm-test")).toThrow("invalid or unverified wasm build product");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
