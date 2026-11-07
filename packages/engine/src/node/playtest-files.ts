// How a playtest file (`tests/*.og.ts`, a bun:test file) is run: `bun test`
// on that one file with the preload that refuses a test without an
// assertion, `@origingame/engine/*` resolved to this package, and the
// runner's settings in the environment, read by run-playtest.ts.
//
// `collectPlaytests` runs a file the same way with `OG_PLAYTEST_COLLECT`
// set: each runPlaytest call records its PlaytestDesc and plays nothing,
// for the tools that read a playtest's script without running the game
// (og ui check's default screens, the web reference page).

import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { PlaytestDesc } from "../playtest.ts";

declare const Bun: {
  spawn(cmd: string[], o: { cwd?: string; env?: Record<string, string | undefined>; stdin?: "ignore"; stdout?: "pipe" | "inherit"; stderr?: "pipe" | "inherit" }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

const here = dirname(fileURLToPath(import.meta.url));
const ENGINE_ROOT = resolve(here, "..", "..");

/** The default per-test timeout of a playtest file: a test runs whole games, not a unit. */
export const PLAYTEST_TIMEOUT_MS = 600_000;

/**
 * A tsconfig for `bun test --tsconfig-override`: `@origingame/engine[/sub]` resolved to this
 * package's exports, as the native bundle resolves them, whatever the game's own paths say.
 */
export function engineTsconfig(path: string): string {
  type Exports = Record<string, string | { default?: string }>;
  const exports = (JSON.parse(readFileSync(join(ENGINE_ROOT, "package.json"), "utf8")) as { exports: Exports }).exports;
  const paths: Record<string, string[]> = {};
  for (const [key, to] of Object.entries(exports)) {
    const file = typeof to === "string" ? to : to.default;
    if (!file || key.endsWith(".json")) continue;
    paths[`@origingame/engine${key.slice(1)}`] = [join(ENGINE_ROOT, file)];
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify({ compilerOptions: { paths } }));
  return path;
}

/** The `bun test` command line for one playtest file, its settings in `env`. */
export function bunTest(file: string, work: string, extra: readonly string[] = []): string[] {
  const timeout = Number(process.env.OG_PLAYTEST_TIMEOUT) > 0 ? process.env.OG_PLAYTEST_TIMEOUT! : String(PLAYTEST_TIMEOUT_MS);
  return ["bun", "test", "--preload", join(here, "playtest-preload.ts"), "--tsconfig-override", engineTsconfig(join(work, ".tsconfig.json")), "--timeout", timeout, ...extra, resolve(file)];
}

/** Every PlaytestDesc `file`'s tests pass to runPlaytest, in call order, without running the game. */
export async function collectPlaytests(file: string, cwd: string = dirname(resolve(file))): Promise<PlaytestDesc[]> {
  const work = join(ENGINE_ROOT, "dist", "playtest-collect", `${process.pid}-${Date.now()}`);
  const log = join(work, "descs.jsonl");
  mkdirSync(work, { recursive: true });
  try {
    writeFileSync(log, "");
    const p = Bun.spawn(bunTest(file, work), { cwd, env: { ...process.env, OG_PLAYTEST_COLLECT: log }, stdin: "ignore", stdout: "pipe", stderr: "pipe" });
    const [code, , stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
    const descs = readFileSync(log, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l) as PlaytestDesc);
    // Every collected call fails its test on purpose; a file that collected nothing and failed did not load.
    if (descs.length === 0 && code !== 0) throw new Error(`${file} does not load: ${stderr.trim()}`);
    return descs;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}
