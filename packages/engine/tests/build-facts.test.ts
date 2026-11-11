// What a build keeps between runs (src/node/web-build.ts: a file's
// compressed size, a host's probe) is kept in the user's cache directory,
// never in the engine's install, and builds side by side keep each other's.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ENGINE_FACTS, factGet, factPut, factsFile } from "../src/node/web-build.ts";

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
