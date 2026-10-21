// A str field is world state through TypeScript on both hosts
// (tests/support/strings-hash.ts, one process per run): a JS string in, a JS
// string out; over the capacity an error value (world.set) or a throw (a
// resource's setter) that names the fix, never truncated; a system's view is
// the zero-padded UTF-8 bytes, stride the capacity; snapshot and restore
// carry the text; the probe's get:/set: read and write it; and two runs end
// on one world hash. The web host (the wasm module, outside a browser) gives
// every one of those answers and the same hash (skipped where the wasm is
// older than the addon, as hosts.test.ts's cross-host tests are).
//
// Ways it could fail, and the check that catches each:
// - the text is truncated or written anyway when over the capacity → "long",
//   "afterLong", "resourceLong", the probe's warning
// - a string's bytes carry anything but the text (padding garbage, a length
//   prefix), so equal names hash apart or a run is not reproducible → the
//   two runs' hashes, "hashRestored"
// - the view's stride is not the capacity → "viewLength", "viewNames"
// - the wasm host packs, pads, views or hashes a string otherwise than the
//   native host → the cross-host test, answer by answer
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the cross-host test skips there and
// runs in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

type Run = Record<string, unknown> & { hash: string; log: string[]; outputs: Array<{ value: unknown }> };

function runOn(host: "node" | "web", ...extra: string[]): Run {
  const r = Bun.spawnSync(["bun", "tests/support/strings-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Run;
}

const run = (...extra: string[]) => runOn("node", ...extra);

test("a str field moves as a JS string, refuses text over its capacity and hashes the same in two runs", () => {
  const a = run();
  const b = run();
  expect(a.got).toEqual([{ name: "Ada", rank: 1 }, { name: "小灯", rank: 2 }, { name: "", rank: 3 }]);
  expect(a.long).toContain("'Bartholomew Bold!' is 17 bytes of UTF-8 and the field holds 16");
  expect(a.long).toContain("declare the field str(17) or larger");
  expect(a.afterLong).toEqual({ name: "Ada", rank: 1 });
  expect(a.full).toBe("ok");
  expect(a.profileDefault).toBe("Mei");
  expect(a.resourceLong).toContain("is 49 bytes of UTF-8 and the field holds 48");
  expect(a.profileAfterLong).toBe("Mei");
  // The view: one 16-byte lane per entity, the text then zeros.
  expect(a.viewCount).toBe(3);
  expect(a.viewLength).toBe(48);
  expect(a.viewNames).toEqual(["小灯笼小灯!", "小灯", ""]);
  // A system's write on tick 5.
  expect(a.profile).toEqual({ name: "小灯 Lantern", renames: 1 });
  // Snapshot and restore.
  expect(a.hashChanged).toBe(true);
  expect(a.restored).toBe(true);
  expect(a.hashRestored).toBe(true);
  expect(a.afterRestore).toEqual({ name: "小灯笼小灯!", rank: 1 });
  // The probe.
  expect(a.outputs.map((o) => o.value)).toEqual(["小灯笼小灯!"]);
  expect(a.probeProfile).toBe("Ada Lovelace");
  expect(a.log.length).toBe(1);
  expect(a.log[0]).toContain("StrBadge.name: 'Bartholomew Bold!' is 17 bytes of UTF-8 and the field holds 16");
  // Deterministic: the same script, the same world.
  expect(b.hash).toBe(a.hash);
  expect(b).toEqual(a);
}, 120_000);

test.skipIf(STALE_WASM)("a str field gives the same answers and the same world hash on the native and the web host", () => {
  const node = runOn("node");
  const web = runOn("web");
  expect(web.host).toBe("web");
  // Every answer, the view's bytes, the probe's outputs and the log's lines included, then the hash.
  expect({ ...web, host: "node" }).toEqual(node);
  expect(web.hash).toBe(node.hash);
}, 120_000);
