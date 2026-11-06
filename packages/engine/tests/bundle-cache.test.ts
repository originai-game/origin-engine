import { expect, test } from "bun:test";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { ogBin } from "./support/og-bin.ts";

declare module "node:fs" {
  export function chmodSync(path: string, mode: number): void;
  export function utimesSync(path: string, atime: number, mtime: number): void;
}

declare const Bun: {
  which(name: string): string | null;
  spawn(args: string[], options: { env: typeof process.env; stdout: "pipe"; stderr: "pipe" }): {
    stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>;
  };
  spawnSync(args: string[], options: { stdout: "pipe"; stderr: "pipe" }): unknown;
};

test.skipIf(process.platform === "win32")("native preparation reuses unchanged inputs and invalidates edits, resolution changes and damaged output", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-bundle-cache-"));
  const root = join(dir, "game");
  mkdirSync(root);
  const log = join(dir, "bundles");
  const bun = Bun.which("bun");
  if (!bun) throw new Error("bun not found");
  // Count actual compiler processes, not a cache's self-reported hit flag.
  writeFileSync(join(dir, "bun"), `#!/bin/sh\ncase "$1" in */src/node/build.ts) echo build >> '${log}';; esac\nexec '${bun}' "$@"\n`);
  chmodSync(join(dir, "bun"), 0o755);
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Cache"\nentry "main.ts"\nseed 83\n');
  const source = 'import {og} from "@origingame/engine";import {count} from "./value";export default og.game({start(w){for(let i=0;i<count;i++)w.spawn({name:"entity"+i});},facts(){return {pid:process.pid};}});';
  writeFileSync(join(root, "main.ts"), source);
  writeFileSync(join(root, "value.ts"), "export const count=1;");
  const env = { ...process.env, PATH: `${dir}:${process.env.PATH}`, OG_ENGINE_PACKAGE: resolve(import.meta.dir!, ".."), OG_RENDERER: "none", OG_SESSION: "1" };
  let hostPid = 0;
  const count = () => {
    try { return readFileSync(log, "utf8").trim().split("\n").length; }
    catch { return 0; }
  };
  const run = async (cold = false) => {
    const child = Bun.spawn([ogBin(), "run", root, "--headless", "--no-check", "--commands", "0.05:quit", "--json"], {
      env: { ...env, OG_SESSION: cold ? "0" : "1" }, stdout: "pipe", stderr: "pipe",
    });
    const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
    expect(code, stdout + stderr).toBe(0);
    const result = JSON.parse(stdout.trim().split("\n").pop()!);
    if (!cold) hostPid = result.facts.pid;
    return result;
  };
  try {
    const initial = await run();
    expect(initial.entities).toBe(1);
    const first = count();
    expect(first).toBe(1);
    expect((await run()).hash).toBe(initial.hash);
    expect(count()).toBe(first);
    expect((await run(true)).hash).toBe(initial.hash);
    const old = statSync(join(root, "value.ts"));
    writeFileSync(join(root, "value.ts"), "export const count=2;");
    utimesSync(join(root, "value.ts"), old.mtimeMs / 1000, old.mtimeMs / 1000);
    const beforePair = count();
    const pair = await Promise.all([run(), run()]);
    expect(pair.map((result) => result.entities)).toEqual([2, 2]);
    expect(pair[0].hash).toBe(pair[1].hash);
    expect(count()).toBe(beforePair + 1);
    expect((await run(true)).hash).toBe(pair[0].hash);
    writeFileSync(join(root, ".og/bundle/bundle.js"), "damaged bundle");
    const beforeDamage = count();
    expect((await run()).entities).toBe(2);
    expect(count()).toBe(beforeDamage + 1);
    rmSync(join(root, "value.ts"));
    writeFileSync(join(root, "value.js"), "export const count=3;");
    expect((await run()).entities).toBe(3);
    writeFileSync(join(root, "value.ts"), "export const count=4;");
    expect((await run()).entities).toBe(4);
    writeFileSync(join(dir, "outside.ts"), "export const count=5;");
    writeFileSync(join(root, "main.ts"), source.replace('"./value"', '"../outside"'));
    const beforeOutside = count();
    expect((await run()).entities).toBe(5);
    expect((await run()).entities).toBe(5);
    expect(count()).toBe(beforeOutside + 2);
    writeFileSync(join(dir, "outside.ts"), "export const count=6;");
    expect((await run()).entities).toBe(6);
    writeFileSync(join(root, "main.ts"), source);
    writeFileSync(join(root, "tsconfig.json"), '{"extends":"../base.json"}');
    writeFileSync(join(dir, "base.json"), "{}");
    const beforeConfig = count();
    expect((await run()).entities).toBe(4);
    expect((await run()).entities).toBe(4);
    expect(count()).toBe(beforeConfig + 2);
  } finally {
    if (hostPid) Bun.spawnSync(["kill", "-TERM", String(hostPid)], { stdout: "pipe", stderr: "pipe" });
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
