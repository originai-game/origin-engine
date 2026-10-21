// A resource handle reads and writes the resource's packed bytes (FINDINGS
// #19 of examples/kestrel-stair): a system writing 200 fields a tick with
// engine calls between the writes gives the world the engine's JSON path
// (one call per field) gave, on both hosts, and a replay from a save
// reaches the same hash. Each run is its own process
// (tests/support/resource-writes.ts).
//
// Ways it could fail, and the check that catches each:
// - a write kept in the runtime's copy and never written back, or written
//   after an engine call that read the resource → the json/bytes hashes,
//   "carriedU1" (a snapshot right after a command's write)
// - a field encoded otherwise than the engine encodes it (f32 rounding, a
//   handle's high word, a string's padding) → the hashes, "values"
// - the web host's bytes cross otherwise → the cross-host case
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const STALE_WASM = process.env.OG_WASM_STALE === "1";

type Run = { host: string; path: string; hash: string; replayed: string; values: Record<string, unknown>; carriedU1: number };

function run(host: "node" | "web", path: "bytes" | "json"): Run {
  const r = Bun.spawnSync(["bun", "tests/support/resource-writes.ts", "--host", host, "--path", path], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} ${path}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Run;
}

test("200 field writes a tick through the byte layout give the world the JSON path gave, and replay from a save", () => {
  const bytes = run("node", "bytes");
  const viaJson = run("node", "json");
  expect(bytes.hash).toBe(viaJson.hash);
  expect(bytes.values).toEqual(viaJson.values);
  expect(bytes.replayed).toBe(bytes.hash);
  expect(viaJson.replayed).toBe(viaJson.hash);
  // The run moved: every kind took a value off its default.
  expect(bytes.values.u0).toBeGreaterThan(0);
  // The last tick of 60 is tick 59.
  expect(bytes.values.s2).toBe("tick 59 · 灯 2");
  expect(bytes.values.h4).toBe((59 * 4294967296 + 4) % 9007199254740991);
  expect(typeof bytes.values.b7).toBe("boolean");
  expect(bytes.values.e1).toBe(["aces", "agx", "none"][(59 + 1) % 3]);
  // A snapshot taken right after a command's write carries it.
  expect(bytes.carriedU1).toBe(424242);
}, 120_000);

test.skipIf(STALE_WASM)("the byte layout gives the same world on the native and the web host", () => {
  const node = run("node", "bytes");
  const web = run("web", "bytes");
  expect(web.hash).toBe(node.hash);
  expect(web.values).toEqual(node.values);
  expect(web.replayed).toBe(node.replayed);
}, 120_000);
