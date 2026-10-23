// A list field is world state through TypeScript on both hosts
// (tests/support/lists-hash.ts, one process per run): a JS array in, a
// frozen array out (a copy, so `push` throws instead of silently changing
// nothing), written whole; over the capacity an error value (world.set) or
// a throw (a resource's setter) that names the fix, never truncated; a
// system's view is the element's typed array with the count first; `[]` and
// `[0]` are different worlds; snapshot and restore carry it; the probe's
// get:/set: read and write it as an array; PostEffects.custom is one; and two
// runs end on one world hash. The web host gives the same answers (skipped
// where the wasm is older than the addon, as strings.test.ts does).
//
// Ways it could fail, and the check that catches each:
// - a read answers the live array, or an unfrozen copy, so `push` silently
//   does nothing → "frozen", "push", "indexWrite", "copy"
// - a list over the capacity is truncated or written anyway → "long",
//   "afterLong", "resourceLong", "afterLongWeights", "customLong"
// - the count lane is missing, so [] and [0] pack alike → "got", "viewLanes",
//   "hashChanged"
// - a handle loses its high word in the byte path → "assets"
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const STALE_WASM = process.env.OG_WASM_STALE === "1";

type Run = Record<string, unknown> & { hash: string; log: string[]; outputs: Array<{ value: unknown }> };

function runOn(host: "node" | "web", ...extra: string[]): Run {
  const r = Bun.spawnSync(["bun", "tests/support/lists-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Run;
}

const run = (...extra: string[]) => runOn("node", ...extra);

test("a list field reads as a frozen copy, is written whole, refuses more than its capacity and hashes the same in two runs", () => {
  const a = run();
  const b = run();
  expect(a.got).toEqual([{ passes: [3, 0], weight: 1 }, { passes: [], weight: 2 }, { passes: [0], weight: 3 }]);
  expect(a.long).toContain("5 elements and the field holds 4");
  expect(a.long).toContain("list(u32(), 5)");
  expect(a.afterLong).toEqual({ passes: [3, 0], weight: 1 });
  // The resource handle: a frozen copy, so push and index writes throw.
  expect(a.defaultWeights).toEqual([]);
  expect(a.frozen).toBe(true);
  expect(a.push).not.toBe("accepted");
  expect(a.indexWrite).not.toBe("accepted");
  expect(a.afterPush).toEqual([]);
  expect(a.written).toEqual([0.5, 2]);
  expect(a.copy).toBe(true);
  expect(a.resourceLong).toContain("ListGrades.weights: 4 elements and the field holds 3");
  expect(a.resourceBadElement).toContain("is not a handle");
  expect(a.afterLongWeights).toEqual([0.5, 2]);
  expect(a.assets).toEqual([4294967298]);
  // PostEffects.custom is a list of eight.
  expect(a.customDefault).toEqual([]);
  expect(a.customFrozen).toBe(true);
  expect(a.customLong).toContain("PostEffects.custom: 9 elements and the field holds 8");
  // The view: five u32 lanes per entity, the count first, zeros past it.
  expect(a.viewCount).toBe(3);
  expect(a.viewLength).toBe(15);
  expect(a.viewLanes).toEqual([[2, 3, 0, 0, 0], [0, 0, 0, 0, 0], [1, 0, 0, 0, 0]]);
  // A system's whole-array write on tick 5.
  expect(a.grades).toEqual({ weights: [0.5, 2, 0.25], assets: [4294967298], turns: 1 });
  // [] is not [0]; snapshot and restore carry the list.
  expect(a.hashChanged).toBe(true);
  expect(a.restored).toBe(true);
  expect(a.hashRestored).toBe(true);
  expect(a.afterRestore).toEqual({ passes: [0], weight: 3 });
  // The probe.
  expect(a.outputs.map((o) => o.value)).toEqual([[3, 0]]);
  expect(a.probeWeights).toEqual([4, 5]);
  expect(a.log).toEqual([]);
  // Deterministic: the same script, the same world.
  expect(b.hash).toBe(a.hash);
  expect(b).toEqual(a);
}, 120_000);

test.skipIf(STALE_WASM)("a list field gives the same answers and the same world hash on the native and the web host", () => {
  const node = runOn("node");
  const web = runOn("web");
  expect(web.host).toBe("web");
  expect({ ...web, host: "node" }).toEqual(node);
  expect(web.hash).toBe(node.hash);
}, 120_000);
