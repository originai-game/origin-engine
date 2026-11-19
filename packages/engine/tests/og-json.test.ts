// The machine-readable output a tool that drives og relies on (the testing
// guide §9): `og test --json`'s events, `og probe`'s result and exit
// status, and `og build --json` with where the build lands. A change to
// any shape asserted here is a breaking change.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { appendFileSync, existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe"; timeout?: number }): { stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>; pid: number; signalCode: string | null; kill(): void };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
// No resident host: these tests pin output shapes, which a warm session does not change.
const env = { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: "none", OG_SESSION: "0" };
let root = "";
let game = "";

const og = (...args: string[]) => {
  const r = Bun.spawnSync([ogBin(), ...args], { cwd: repo, env });
  return { code: r.exitCode, stdout: new TextDecoder().decode(r.stdout), stderr: new TextDecoder().decode(r.stderr) };
};

const BUILD_WALL_MS = 180_000;

async function tracedBuild(name: string, deadline: number, ...args: string[]) {
  const logs = process.env.GATE_LOGS;
  if (!logs) return og("build", ...args);
  const trace = join(logs, `og-json-${name}.jsonl`);
  const child = Bun.spawn([ogBin(), "build", ...args], {
    cwd: repo, env: { ...env, OG_BUILD_TRACE: trace }, stdout: "pipe", stderr: "pipe",
    timeout: Math.max(1, Math.floor(deadline - performance.now())),
  });
  const record = (event: string, fields: Record<string, unknown>) => {
    try {
      appendFileSync(trace, JSON.stringify({ at: Date.now(), pid: child.pid, event, ...fields }) + "\n");
    } catch {
      // The contract is independent of evidence storage.
    }
  };
  record("spawn", {});
  const pipe = async (stream: "stdout" | "stderr") => {
    const reader = child[stream].getReader();
    const decoder = new TextDecoder();
    let output = "";
    for (;;) {
      const { done, value } = await reader.read();
      const chunk = done ? decoder.decode() : decoder.decode(value, { stream: true });
      if (chunk) record(stream, { chunk });
      output += chunk;
      if (done) return output;
    }
  };
  const sample = setInterval(() => {
    if (process.platform === "win32") return;
    const result = Bun.spawnSync(["ps", "-axo", "pid=,ppid=,stat=,pcpu=,rss=,time=,comm="], { cwd: repo, env });
    const rows = new TextDecoder().decode(result.stdout).trim().split("\n").map((row) => row.trim());
    const family = new Set([String(child.pid)]);
    for (let size = 0; size !== family.size;) {
      size = family.size;
      for (const row of rows) {
        const [pid, parent] = row.split(/\s+/, 3);
        if (pid && parent && family.has(parent)) family.add(pid);
      }
    }
    record("process-tree", { rows: rows.filter((row) => family.has(row.split(/\s+/, 1)[0]!)) });
  }, 15_000);
  try {
    const [code, stdout, stderr] = await Promise.all([child.exited, pipe("stdout"), pipe("stderr")]);
    record("exit", { status: code, signal: child.signalCode });
    return { code, stdout, stderr };
  } finally {
    clearInterval(sample);
  }
}
/** Every stdout line, each of which must be one JSON object. */
const lines = (stdout: string) => stdout.split("\n").filter((l) => l !== "").map((l) => JSON.parse(l) as Record<string, unknown>);

beforeAll(() => {
  root = mkdtempSync(join(tmpdir(), "og-json-"));
  game = join(root, "game");
  const created = Bun.spawnSync(["bun", join(repo, "packages/create/bin/create.ts"), game, "--name", "Json", "--no-prompt"], { cwd: repo, env });
  if (created.exitCode !== 0) throw new Error(new TextDecoder().decode(created.stderr));
  writeFileSync(join(game, "src/main.ts"), `import { og } from "@origingame/engine"; export default og.game({});`);
});
afterAll(() => rmSync(root, { recursive: true, force: true }));

test("og test --json prints only JSON Lines: check, one playtest per file in order, then test", () => {
  writeFileSync(join(game, "unit.test.ts"), `import { expect, test } from "bun:test"; test("unit", () => expect(1).toBe(1));`);
  const playtest = (ok: boolean) => `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("quits", async () => { expect((await runPlaytest({ commands: "0.1:quit" })).quit, "quit").toBe(${ok}); });`;
  writeFileSync(join(game, "tests/a.og.ts"), playtest(true));
  writeFileSync(join(game, "tests/b.og.ts"), playtest(false));
  rmSync(join(game, "tests/smoke.og.ts"), { force: true });
  const r = og("test", game, "--json");
  const events = lines(r.stdout);
  expect(events.map((e) => [e.event, e.playtest])).toEqual([["check", undefined], ["playtest", "a"], ["playtest", "b"], ["test", undefined]]);
  const [check, a, b, last] = events as [Record<string, any>, Record<string, any>, Record<string, any>, Record<string, any>];
  expect(check).toMatchObject({ ok: true, diagnostics: [] });
  expect(typeof check.ms).toBe("number");
  expect(a).toMatchObject({ ok: true, status: "passed", tests: 1, checks: 1, failures: [], uncheckedCount: 0, out: join(game, "og-out/test/a") });
  expect(b).toMatchObject({ ok: false, status: "failed", tests: 1 });
  expect(b.failures.join("\n")).toContain("quit");
  for (const key of ["file", "frameChecks", "unchecked", "failureCaptures", "ms", "assertions", "runs", "adapterErrors", "hash", "tier"]) expect(a).toHaveProperty(key);
  expect(last).toMatchObject({ ok: false, suites: 3, passed: 2, failed: 1, uncheckedSuites: 0, uncheckedCount: 0, skipped: 0, check: { ok: true, errors: 0 } });
  expect(r.code).toBe(1);
  // The bun tests' report and the summary are on stderr.
  expect(r.stderr).toContain("og test: 2 suite(s) passed, 1 failed");
  rmSync(join(game, "unit.test.ts"));
  rmSync(join(game, "tests/b.og.ts"));
}, 180_000);

test("og probe prints the result as JSON, exits 1 on an error, and --raw prints the frame", async () => {
  const port = String(40000 + (process.pid % 20000));
  const child = Bun.spawn([ogBin(), "run", game, "--headless", "--no-check", "--port", port], { cwd: repo, env, stdout: "pipe", stderr: "pipe" });
  try {
    const reader = child.stdout.getReader();
    let out = "";
    while (!out.includes("probe: ws://")) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og run ended before the probe was up: ${out}${await new Response(child.stderr).text()}`);
      out += new TextDecoder().decode(value);
    }
    const hello = og("probe", "hello", "--port", port);
    expect(hello.code).toBe(0);
    const result = JSON.parse(hello.stdout);
    expect(result.members).toContain("hello");
    expect(typeof result.engine).toBe("string");
    // A refusal is a result.
    const refused = og("probe", "entity", '{"id":"no-such-entity"}', "--port", port);
    expect([refused.code, JSON.parse(refused.stdout)]).toEqual([0, null]);
    const unknown = og("probe", "no_such_member", "--port", port);
    expect([unknown.code, unknown.stdout]).toEqual([1, ""]);
    expect(unknown.stderr).toContain("og probe: no_such_member: error -32601");
    const raw = JSON.parse(og("probe", "tick", "--port", port, "--raw").stdout);
    expect(raw).toMatchObject({ jsonrpc: "2.0", id: 1 });
    expect(typeof raw.result).toBe("number");
    // The game ends through its probe; it then no longer answers.
    expect(og("probe", "commands", '{"script":"0:quit"}', "--port", port).code).toBe(0);
    expect(await child.exited).toBe(0);
  } finally {
    child.kill();
    await child.exited;
  }
  const gone = og("probe", "hello", "--port", port);
  expect([gone.code, gone.stdout]).toEqual([1, ""]);
}, 120_000);

test("og build --json prints check and build events; the build lands in dist/ or --out", async () => {
  const deadline = performance.now() + BUILD_WALL_MS;
  const events = (r: { stdout: string }) => lines(r.stdout);
  const r = await tracedBuild("default", deadline, game, "--json");
  expect(r.code).toBe(0);
  const [check, build] = events(r) as [Record<string, any>, Record<string, any>];
  expect(events(r).map((e) => e.event)).toEqual(["check", "build"]);
  expect(check.ok).toBe(true);
  expect(build).toMatchObject({ ok: true, out: join(game, "dist"), publish: false });
  for (const key of ["files", "total", "downloads", "probe"]) expect(build).toHaveProperty(key);
  expect(build.manifest.entry).toBe("index.html");
  expect(existsSync(join(game, "dist", "og-build.json"))).toBe(true);
  expect(existsSync(join(game, "dist", build.manifest.entry))).toBe(true);
  const elsewhere = join(root, "elsewhere");
  const moved = await tracedBuild("elsewhere", deadline, game, "--json", "--no-check", "--out", elsewhere);
  expect(moved.code).toBe(0);
  expect(events(moved).map((e) => [e.event, e.out])).toEqual([["build", elsewhere]]);
  expect(existsSync(join(elsewhere, "og-build.json"))).toBe(true);
}, BUILD_WALL_MS);
