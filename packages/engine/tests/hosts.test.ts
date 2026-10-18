// The two hosts are one engine: the design's hello world (10,000 entities,
// one Move system) gives the same world hash after 600 ticks on the native
// addon and on the wasm module. Each host runs in its own process.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

function hello(host: string, ...extra: string[]): { tick: number; hash: string; hashAtLoad: string; entities: number } {
  const r = Bun.spawnSync(["bun", "tests/support/hello-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
}

test("the native and the web host agree on the world hash after 600 ticks", () => {
  const node = hello("node");
  const web = hello("web");
  expect(node.tick).toBe(600);
  expect(node.entities).toBe(10000);
  expect(web.hash).toBe(node.hash);
  // The hash measures the simulation: it moved from load, and a different
  // seed (other spawn positions) gives another world.
  expect(node.hash).not.toBe(node.hashAtLoad);
  expect(hello("node", "--seed", "7").hash).not.toBe(node.hash);
}, 120_000);

test("a probe argument of the wrong type is refused alike on both hosts, and the engine keeps running", () => {
  const run = (host: string) => {
    const r = Bun.spawnSync(["bun", "tests/support/probe-refusal.ts", "--host", host], { cwd: pkg });
    const out = new TextDecoder().decode(r.stdout);
    if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
    return JSON.parse(out.trim().split("\n").pop()!) as { refused: string; scheduled: number; tick: number; hash: string };
  };
  const [node, web] = [run("node"), run("web")];
  expect(web.refused).toBe("probe.commands: script must be a string, not object");
  expect(node.refused).toBe(web.refused);
  // After the refusal the same engine schedules, ticks and hashes as usual.
  expect(web.scheduled).toBe(1);
  expect(web.tick).toBe(30);
  expect(web.hash).toBe(node.hash);
}, 120_000);

function physics(host: string, ...extra: string[]) {
  const r = Bun.spawnSync(["bun", "tests/support/physics-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as {
    tick: number; hash: string; hashAtLoad: string; body63: { y: number } | null; tally: { entered: number; started: number } | null;
  };
}

for (const profile of ["3d", "2d"]) {
  test(`a ${profile} physics scene gives the same world hash on both hosts after 600 ticks`, () => {
    const node = physics("node", "--profile", profile);
    const web = physics("web", "--profile", profile);
    expect(node.tick).toBe(600);
    expect(web.hash).toBe(node.hash);
    expect(web.body63).toEqual(node.body63);
    // The simulation ran: bodies fell and settled above the floor, and the
    // collision and sensor events reached a system.
    expect(node.hash).not.toBe(node.hashAtLoad);
    expect(node.body63!.y).toBeGreaterThan(0);
    expect(node.body63!.y).toBeLessThan(6);
    expect(node.tally!.started).toBeGreaterThan(0);
    expect(node.tally!.entered).toBeGreaterThan(0);
  }, 180_000);
}
