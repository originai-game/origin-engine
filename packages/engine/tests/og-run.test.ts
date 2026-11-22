// `og run --headless` loads the hello world on the native host and prints
// the same world hash the web host computes.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { ogBin } from "./support/og-bin.ts";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe" }): { stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>; kill(signal?: string): void };
};

const spawn = (cmd: string[], cwd: string) => {
  const r = Bun.spawnSync(cmd, { cwd });
  return { exitCode: r.exitCode, stdout: new TextDecoder().decode(r.stdout), stderr: new TextDecoder().decode(r.stderr) };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

const run = (cmd: string[], cwd: string) => {
  const r = Bun.spawnSync(cmd, { cwd });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
};

test.skipIf(STALE_WASM)("og run --headless prints the hello world's hash after 600 ticks", () => {
  const og = run(
    [ogBin(), "run", "--headless", "packages/engine/examples/hello.ts", "--ticks", "600", "--setup", "spawn_wave:10000", "--json"],
    repo,
  );
  const web = run(["bun", "tests/support/hello-hash.ts", "--host", "web"], pkg);
  expect(og.tick).toBe(600);
  expect(og.entities).toBe(10000);
  expect(og.hash).toBe(web.hash);
}, 300_000);

test("a throw in GameDesc.start fails og run, named with its stack, with and without a command script", () => {
  const game = "packages/engine/tests/fixtures/start-throws/main.ts";
  for (const args of [["--ticks", "10"], ["--commands", "0.1:quit"], ["--commands", "0.1:quit", "--no-render"]]) {
    const r = spawn([ogBin(), "run", "--headless", game, ...args], repo);
    if (r.exitCode === 0) throw new Error(`og run ${args.join(" ")} exited 0: ${r.stderr}${r.stdout}`);
    expect(r.stderr).toContain("GameDesc.start threw");
    expect(r.stderr).toContain("the village has no well");
    expect(r.stderr).toContain("buildTheVillage");
  }
}, 120_000);

test("og run takes tick and seed from project.ogt for a GameDesc that leaves them unset", () => {
  const out = mkdtempSync(join(tmpdir(), "og-project-"));
  try {
    const r = spawn([ogBin(), "run", "--headless", "packages/engine/tests/fixtures/project-defaults", "--no-render", "--out", out, "--commands", "0:get:og.Time:time.json;0:get:og.Rng.seed:seed.json;0.1:quit"], repo);
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${r.stderr}${r.stdout}`);
    const read = (f: string) => (JSON.parse(readFileSync(join(out, f), "utf8")) as { value: unknown }).value;
    expect((read("time.json") as { rate: number }).rate).toBe(30);
    expect(read("seed.json")).toBe(5);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);

test("run and replay read command files larger than a process argument, relative to the caller", () => {
  const out = mkdtempSync(join(tmpdir(), "og-command-file-"));
  try {
    const script = "0:set:og.Audio.master=1;".repeat(6_000) + "0.1:quit";
    expect(script.length).toBeGreaterThan(131_072);
    writeFileSync(join(out, "commands.txt"), `\n  ${script}\n`);
    const game = join(repo, "packages/engine/tests/fixtures/project-defaults");
    const args = ["--commands", "0:quit", "--commands-file", "commands.txt", "--json"];
    const summary = run([ogBin(), "run", game, "--headless", "--no-render", ...args], out);
    expect([summary.tick, summary.quit]).toEqual([3, true]);
    // Reverse the options: the inline script wins instead.
    const inline = run([ogBin(), "run", game, "--headless", "--no-render", "--commands-file", "commands.txt", "--commands", "0.2:quit", "--json"], out);
    expect(inline.tick).toBe(6);
    const trace = join(out, "trace.json");
    const recorded = spawn([ogBin(), "replay", "record", game, ...args, "--out", trace], out);
    expect(recorded.exitCode, recorded.stderr).toBe(0);
    expect(JSON.parse(readFileSync(trace, "utf8")).commands).toBe(script);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);

test("a restore that moves og.Time past the script's last scheduled tick still runs the steps after it, and quit ends the run", () => {
  // brasswick FINDINGS #13: the run ended at the first tick past the tick
  // its last step was scheduled for, so a Continue restoring a later save
  // ended it at once, without the steps after it, a quit or a warning.
  const game = "packages/engine/tests/fixtures/restore-forward";
  const first = spawn([ogBin(), "run", "--headless", game, "--no-render", "--commands", "5:print_save;5.1:quit"], repo);
  if (first.exitCode !== 0) throw new Error(`og run exited ${first.exitCode}: ${first.stderr}${first.stdout}`);
  const save = first.stdout.split("\n").find((l) => l.startsWith("og-save:3:"));
  if (!save) throw new Error(`no save printed: ${first.stdout}`);
  const out = mkdtempSync(join(tmpdir(), "og-restore-forward-"));
  try {
    // The command at tick 3 restores the save of tick 150 before tick 3 moves anything, so tick 150
    // runs in its place: the get (due at 15) and the quit (due at 18) keep the 12 and 15 ticks they had left.
    const r = spawn([ogBin(), "run", "--headless", game, "--no-render", "--out", out, "--json", "--commands", `0.1:load:${save};0.5:get:og.Time.tick:tick.json;0.6:quit`], repo);
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${r.stderr}${r.stdout}`);
    const summary = JSON.parse(r.stdout.trim().split("\n").pop()!) as { tick: number; quit: boolean };
    expect(summary.quit).toBeTrue();
    expect(existsSync(join(out, "tick.json"))).toBeTrue();
    expect((JSON.parse(readFileSync(join(out, "tick.json"), "utf8")) as { value: number }).value).toBe(150 + 12);
    expect(summary.tick).toBe(150 + 15);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);

test("a save a command takes and restores at once leaves the world's tick and hash as they were", () => {
  // A command runs between ticks, so its restore applies before the next tick moves anything; one
  // applied as that tick ended undid it, leaving the run a tick behind (flow guide §8).
  const game = "packages/engine/tests/fixtures/restore-forward";
  const plain = run([ogBin(), "run", "--headless", game, "--no-render", "--json", "--commands", "1:quit"], repo);
  const restored = run([ogBin(), "run", "--headless", game, "--no-render", "--json", "--commands", "0.5:roundtrip;1:quit"], repo);
  expect(restored.tick).toBe(plain.tick);
  expect(restored.hash).toBe(plain.hash);
}, 120_000);

test("og run --profile writes the CPU profile at the path named, its locations in the game's source", () => {
  const out = mkdtempSync(join(tmpdir(), "og-profile-"));
  try {
    const md = join(out, "game.md");
    const r = spawn([ogBin(), "run", "--headless", "packages/engine/examples/hello.ts", "--no-render", "--ticks", "300", "--setup", "spawn_wave:10000", "--profile", md], repo);
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${r.stderr}${r.stdout}`);
    // The bundle is what runs; its source map is what names hello.ts.
    const profile = readFileSync(md, "utf8");
    expect(profile).toContain("Hot Functions");
    expect(profile).toMatch(/examples\/hello\.ts:\d+/);
    expect(profile).not.toContain("game.native.js");
    const device = spawn([ogBin(), "run", "--device", "chrome", "packages/engine/examples/hello.ts", "--profile", md], repo);
    expect(device.exitCode).not.toBe(0);
    expect(device.stderr).toContain("--profile is a native run's option");
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);

test("the game process and its probe end with og run, whether og is terminated or killed", async () => {
  const open = (port: number) => new Promise<boolean>((done) => {
    const socket = new WebSocket(`ws://127.0.0.1:${port}/probe`);
    socket.onopen = () => (socket.close(), done(true));
    socket.onerror = () => done(false);
  });
  for (const [i, signal] of (["SIGTERM", "SIGKILL"] as const).entries()) {
    const port = 41000 + (process.pid % 10000) + i;
    const og = Bun.spawn([ogBin(), "run", "--headless", "packages/engine/tests/fixtures/project-defaults", "--no-render", "--no-check", "--port", String(port)], { cwd: repo, env: { ...process.env, OG_RENDERER: "none" }, stdout: "pipe", stderr: "pipe" });
    try {
      const reader = og.stdout.getReader();
      let out = "";
      while (!out.includes("probe: ws://")) {
        const { value, done } = await reader.read();
        if (done) throw new Error(`og run ended before its probe: ${out}${await new Response(og.stderr).text()}`);
        out += new TextDecoder().decode(value);
      }
      expect(await open(port)).toBe(true);
      // On SIGKILL og cleans nothing up: the game process must end without its help.
      og.kill(signal);
      let closed = false;
      for (let t = 0; t < 50 && !closed; t++) {
        closed = !(await open(port));
        if (!closed) await new Promise((r) => setTimeout(r, 100));
      }
      expect(closed, `the probe on ${port} still answers 5 s after og run got ${signal}`).toBe(true);
    } finally {
      og.kill("SIGKILL");
      await og.exited;
    }
  }
}, 60_000);
