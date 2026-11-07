import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createConnection, createServer } from "node:net";
import { bundleGame } from "../src/node/build.ts";
import { run } from "../src/node/run.ts";
import { runInSession } from "../src/node/session-client.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(args: string[], options: { stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(args: string[], options: { env: typeof process.env; stdout: "ignore" | "pipe"; stderr: "pipe" }): {
    pid: number;
    stdout: ReadableStream<Uint8Array>;
    stderr: ReadableStream<Uint8Array>;
    exitCode: number | null;
    exited: Promise<number>;
    kill(signal: string): void;
  };
};

test("fresh runs isolate module state, world mutations and failed setup", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-test-"));
  const source = join(dir, "main.ts");
  const entry = join(dir, "game.js");
  const runner = resolve(import.meta.dir!, "../src/node/run.ts");
  try {
    writeFileSync(source, `import { og } from "@origingame/engine";
let starts = 0;
export default og.game({
  start(world) { starts++; for (let i=0; i<starts; i++) world.spawn({name:"initial-"+i}); },
  commands: { mutate(world) { starts += 20; world.spawn({name:"contamination"}); } },
  facts() { return { starts }; }
});`);
    const bundled = await bundleGame(source, entry, "native");
    if (!bundled.ok) throw new Error(bundled.reason);
    const args = (script: string) => ["--entry", entry, "--no-render", "--seed", "83", "--clock", "1767225600000", "--commands", script, "--json", "--out", dir];
    const script = "0.2:dump_world:world.json;0.3:quit";
    const cold = Bun.spawnSync(["bun", runner, ...args(script)], { stdout: "pipe", stderr: "pipe" });
    expect(cold.exitCode, new TextDecoder().decode(cold.stderr)).toBe(0);
    const signature = (stdout: string) => {
      const value = JSON.parse(stdout.trim().split("\n").pop()!);
      return {
        hash: value.hash, tick: value.tick, entities: value.entities, facts: value.facts, audio: value.audio,
        assertions: { violations: value.assertions.violations, sites: value.assertions.sites, uncovered: value.assertions.uncovered },
      };
    };
    const expected = signature(new TextDecoder().decode(cold.stdout));
    expect(expected.entities).toBe(1);
    expect(expected.facts.starts).toBe(1);
    const invoke = async (argv: string[]) => {
      let stdout = "";
      let stderr = "";
      const code = await run(argv, { stdout: { write: (s) => { stdout += s; } }, stderr: { write: (s) => { stderr += s; } } });
      return { code, stdout, stderr };
    };
    for (let i = 0; i < 3; i++) {
      const actual = await invoke(args(script));
      expect(actual.code).toBe(0);
      expect(signature(actual.stdout)).toEqual(expected);
      const mutation = await invoke(args("0.1:mutate;0.3:quit"));
      expect(mutation.code).toBe(0);
      expect(signature(mutation.stdout).entities).toBe(2);
      expect(signature(mutation.stdout).facts.starts).toBe(21);
      const failed = await invoke([...args(script), "--setup", "not_a_command"]);
      expect(failed.code).toBe(2);
    }
    writeFileSync(source, '\n\nthrow new Error("mapped failure");\nexport default {};\n');
    const failingBundle = await bundleGame(source, entry, "native");
    if (!failingBundle.ok) throw new Error(failingBundle.reason);
    const stack = await invoke(args(script)).catch((error: Error) => error.stack);
    expect(stack).toContain(`${source}:3:`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);

test.skipIf(process.platform === "win32")("RPC never retries an accepted request after a lost or contradictory reply", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-wire-"));
  const path = join(dir, "host.sock");
  let reply = "";
  const replies: string[] = [];
  let requests = 0;
  const server = createServer((socket) => {
    let received = false;
    socket.on("data", () => {
      if (received) return;
      received = true;
      requests++;
      socket.end(replies.shift() ?? reply);
    });
    socket.on("error", () => socket.destroy());
  });
  await new Promise<void>((resolve) => server.listen(path, resolve));
  try {
    reply = '{"event":"accepted"}\n{"event":"stdout","text":"one"}\n{"event":"exit","code":7}\n';
    expect(await runInSession([], path, "test")).toEqual({ exitCode: 7, stdout: "one", stderr: "" });
    replies.push('{"event":"busy"}\n{"event":"exit","code":75}\n');
    const beforeBusy = requests;
    expect(await runInSession([], path, "test")).toEqual({ exitCode: 7, stdout: "one", stderr: "" });
    expect(requests - beforeBusy).toBe(2);
    reply = '{"event":"accepted"}\n{"event":"busy"}\n';
    const beforeAccepted = requests;
    expect(await runInSession([], path, "test").catch((error: Error) => error.message)).toContain("invalid session reply");
    expect(requests - beforeAccepted).toBe(1);
    reply = '{"event":"accepted"}\n{"event":"stdout","text":"partial side effect"}\n';
    expect(await runInSession([], path, "test").catch((error: Error) => error.message)).toContain("outcome unknown");
    reply = '{"event":"accepted"}\n{"event":"rejected","reason":"too late"}\n';
    expect(await runInSession([], path, "test").catch((error: Error) => error.message)).toContain("invalid session reply");
    reply = '{"event":"rejected","reason":"different build"}\n';
    expect(await runInSession([], path, "test")).toBeNull();
  } finally {
    server.close();
    rmSync(dir, { recursive: true, force: true });
  }
});

test.skipIf(process.platform === "win32")("resident admission freezes queued games, bounds the queue and recovers from cancellation", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-queue-"));
  const path = join(dir, "host.sock");
  const entry = join(dir, "game.ts");
  const engine = resolve(import.meta.dir!, "../src/index.ts");
  const game = (count: number) => `import { og } from ${JSON.stringify(engine)};
export default og.game({start(world){for(let i=0;i<${count};i++)world.spawn({name:"entity"+i});}});`;
  writeFileSync(entry, game(1));
  const env = { ...process.env, OG_RENDERER: "none" };
  const host = Bun.spawn(["bun", resolve(import.meta.dir!, "../src/node/session.ts"), path, "test"], { env, stdout: "ignore", stderr: "pipe" });
  const sockets = new Set<ReturnType<typeof createConnection>>();
  const request = (ticks: number) => {
    let admit!: (value: boolean) => void;
    const accepted = new Promise<boolean>((resolve) => { admit = resolve; });
    let finish!: (value: { code: number; stdout: string }) => void;
    const result = new Promise<{ code: number; stdout: string }>((resolve) => { finish = resolve; });
    let input = "", stdout = "";
    const socket = createConnection(path, () => socket.write(JSON.stringify({
      cwd: process.cwd(), env, build: "test", argv: ["--entry", entry, "--ticks", String(ticks), "--no-render", "--json"],
    }) + "\n"));
    sockets.add(socket);
    socket.setEncoding("utf8");
    socket.on("error", () => socket.destroy());
    socket.on("close", () => {
      sockets.delete(socket);
      admit(false);
      finish({ code: -1, stdout });
    });
    socket.on("data", (text) => {
      input += text;
      while (input.includes("\n")) {
        const i = input.indexOf("\n");
        const line = JSON.parse(input.slice(0, i));
        input = input.slice(i + 1);
        if (line.event === "accepted") admit(true);
        if (line.event === "stdout") stdout += line.text;
        if (line.event === "exit") {
          admit(false);
          finish({ code: line.code, stdout });
          socket.end();
        }
      }
    });
    return { accepted, result, socket };
  };
  try {
    expect(new TextDecoder().decode((await host.stderr.getReader().read()).value)).toContain("listening");
    const blocker = request(100_000);
    expect(await blocker.accepted).toBeTrue();
    const cancelled = request(2);
    expect(await cancelled.accepted).toBeTrue();
    const queued = [];
    for (let i = 0; i < 6; i++) {
      const call = request(2);
      expect(await call.accepted).toBeTrue();
      queued.push(call);
    }
    const refused = request(2);
    expect(await refused.accepted).toBeFalse();
    expect((await refused.result).code).toBe(75);
    writeFileSync(entry, game(3));
    cancelled.socket.destroy();
    blocker.socket.destroy();
    for (const call of queued) {
      const result = await call.result;
      expect(result.code).toBe(0);
      expect(JSON.parse(result.stdout.trim()).entities).toBe(1);
    }
    const changed = await request(2).result;
    expect(changed.code).toBe(0);
    expect(JSON.parse(changed.stdout.trim()).entities).toBe(3);
    const interrupted = request(100_000);
    expect(await interrupted.accepted).toBeTrue();
    host.kill("SIGKILL");
    expect((await interrupted.result).code).toBe(-1);
  } finally {
    for (const socket of sockets) socket.destroy();
    if (host.exitCode === null) host.kill("SIGTERM");
    await host.exited;
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);

test.skipIf(process.platform === "win32")("resident recovers from exceptions and output limits, then retires its module cache", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-retire-"));
  const path = join(dir, "host.sock");
  const entry = join(dir, "game.ts");
  const engine = resolve(import.meta.dir!, "../src/index.ts");
  const memory = join(dir, "memory.ts");
  writeFileSync(memory, 'import { mock } from "bun:test"; import * as os from "node:os"; mock.module("node:os", () => ({ ...os, freemem: () => 0 }));');
  const host = Bun.spawn(["bun", "--preload", memory, resolve(import.meta.dir!, "../src/node/session.ts"), path, "test"], { env: process.env, stdout: "ignore", stderr: "pipe" });
  const argv = ["--entry", entry, "--ticks", "1", "--no-render", "--json"];
  try {
    const reader = host.stderr.getReader();
    expect(new TextDecoder().decode((await reader.read()).value)).toContain("listening");
    writeFileSync(entry, 'throw new Error("module exception");');
    const failed = await runInSession(argv, path, "test");
    expect(failed?.exitCode).toBe(1);
    expect(failed?.stderr).toContain("module exception");
    writeFileSync(entry, 'console.log("x".repeat(17 * 1024 * 1024));');
    const overflow = await runInSession(argv, path, "test");
    expect(overflow?.exitCode).toBe(1);
    expect(overflow?.stderr).toContain("output exceeds 16 MiB");
    writeFileSync(entry, `import { og } from ${JSON.stringify(engine)};
let starts = 0; export default og.game({start(world){starts++;world.spawn({name:"fresh"});},facts(){return {starts};}});`);
    const results = [];
    for (let i = 2; i < 127; i++) {
      const result = await runInSession(argv, path, "test");
      if (result === null) throw new Error(`host stopped at admission ${i}: ${new TextDecoder().decode((await reader.read()).value)}`);
      const summary = JSON.parse(result?.stdout.trim() ?? "null");
      results.push([result?.exitCode, summary.entities, summary.facts.starts]);
    }
    expect(results).toEqual(Array.from({ length: 125 }, () => [0, 1, 1]));
    writeFileSync(entry, 'console.log("x".repeat(768 * 1024) + "MARKER-END");\n' + readFileSync(entry, "utf8"));
    const lastReply = await new Promise<string>((resolve, reject) => {
      let wire = "";
      const socket = createConnection(path, () => {
        socket.pause();
        socket.write(JSON.stringify({ cwd: process.cwd(), env: process.env, build: "test", argv }) + "\n");
        setTimeout(() => socket.resume(), 200);
      });
      socket.setEncoding("utf8");
      socket.on("data", (chunk) => { wire += chunk; });
      socket.on("error", reject);
      socket.on("end", () => resolve(wire));
    });
    const events = lastReply.trim().split("\n").map((line) => JSON.parse(line));
    expect(events.at(-1)).toEqual({ event: "exit", code: 0 });
    const stdout = events.filter((event) => event.event === "stdout").map((event) => event.text).join("");
    expect(stdout).toContain("x".repeat(768 * 1024) + "MARKER-END");
    expect(JSON.parse(stdout.trim().split("\n").at(-1)!).facts.starts).toBe(1);
    expect(await host.exited).toBe(75);
  } finally {
    if (host.exitCode === null) host.kill("SIGTERM");
    await host.exited;
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);

test.skipIf(process.platform === "win32")("concurrent playtests share fresh runs, repeats and then steps in one host", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-playtests-"));
  const path = join(dir, "host.sock");
  const entry = join(dir, "game.ts");
  const executions = join(dir, "executions.txt");
  const engine = resolve(import.meta.dir!, "../src/index.ts");
  const env = { ...process.env, OG_RENDERER: "none" };
  writeFileSync(entry, `import { og } from ${JSON.stringify(engine)};
import { appendFileSync } from "node:fs";
appendFileSync(${JSON.stringify(executions)}, process.pid + "\\n");
let starts = 0;
export default og.game({start(world){starts++;for(let i=0;i<starts;i++)world.spawn({name:"fresh"+i});}});`);
  const host = Bun.spawn(["bun", resolve(import.meta.dir!, "../src/node/session.ts"), path, "test"], { env, stdout: "ignore", stderr: "pipe" });
  const invoke = async (name: string, source: string) => {
    const file = join(dir, `${name}.og.ts`);
    writeFileSync(file, source);
    const child = Bun.spawn(["bun", resolve(import.meta.dir!, "../src/node/playtest-run.ts"),
      "--file", file, "--entry", entry, "--out", join(dir, name), "--json", "--session-socket", path, "--session-build", "test"],
      { env, stdout: "pipe", stderr: "pipe" });
    const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
    return { code, summary: JSON.parse(stdout.trim().split("\n").at(-1)!), stderr };
  };
  const descriptor = (entities: number) => `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("fresh runs", async () => {
const run = await runPlaytest({ commands: "0.1:quit", runs: 2, then: [{ commands: "0.1:quit" }] });
expect(run.entities).toBe(${entities}); expect(run.hashes[1]).toBe(run.hashes[0]); expect(run.then[0].hash).toBe(run.hash);
});`;
  try {
    expect(new TextDecoder().decode((await host.stderr.getReader().read()).value)).toContain("listening");
    for (const result of await Promise.all([invoke("first", descriptor(1)), invoke("second", descriptor(1))])) {
      expect(result.code, JSON.stringify(result.summary) + result.stderr).toBe(0);
      expect(result.summary.checks).toBe(3);
      expect(result.summary.uncheckedCount).toBe(0);
      expect(result.summary.failureCaptures).toEqual([]);
      expect(existsSync(join(result.summary.out, ".failure"))).toBeFalse();
      expect(result.stderr).not.toContain("running alone");
    }
    expect(readFileSync(executions, "utf8").trim().split("\n")).toEqual(Array(6).fill(String(host.pid)));
    const failed = await invoke("first", descriptor(2));
    expect(failed.code).toBe(1);
    expect(failed.summary.failures.length).toBe(1);
    expect(failed.summary.failureCaptures.length).toBe(3);
    for (const capture of failed.summary.failureCaptures) {
      const world = JSON.parse(readFileSync(capture.world, "utf8"));
      expect(world.hash).toBe(failed.summary.hash);
      expect(world.tick).toBe(6);
      expect(readFileSync(capture.snapshot, "utf8")).toMatch(/^og-save:/);
      expect(capture.frame.status).toBe("unavailable");
      expect(existsSync(join(capture.run, ".failure/frame.rgba"))).toBeFalse();
    }
    expect((await invoke("first", descriptor(1))).code).toBe(0);
    const malformed = await invoke("malformed", "export default { invalid syntax");
    expect(malformed.code).toBe(1);
    expect(malformed.summary.failures[0]).toContain("does not load");
  } finally {
    if (host.exitCode === null) host.kill("SIGTERM");
    await host.exited;
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);

test.skipIf(process.platform === "win32")("CLI races auto-start one resident for bare entries and preserves the cold opt-out", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-autostart-"));
  const entry = join(dir, "main.ts");
  const pidFile = join(dir, "host.pid");
  const env = { ...process.env, OG_RENDERER: "none", OG_ENGINE_PACKAGE: resolve(import.meta.dir!, ".."), OG_SESSION: "1" };
  writeFileSync(entry, `import { og } from "@origingame/engine";
import { writeFileSync } from "node:fs";
writeFileSync(${JSON.stringify(pidFile)},String(process.pid));
let starts=0;export default og.game({start(w){starts++;w.spawn({name:"fresh"});},facts(){return {pid:process.pid,starts};}});`);
  const invoke = async (cold = false) => {
    const child = Bun.spawn([ogBin(), "run", entry, "--headless", "--no-check", "--no-render", "--ticks", "2", "--json"],
      { env: { ...env, ...(cold ? { OG_SESSION: "0" } : {}) }, stdout: "pipe", stderr: "pipe" });
    const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
    expect(code, stdout + stderr).toBe(0);
    expect(stderr).not.toContain("running alone");
    return JSON.parse(stdout.trim().split("\n").at(-1)!);
  };
  let hostPid: number | undefined;
  try {
    const pair = await Promise.all([invoke(), invoke()]);
    hostPid = pair[0].facts.pid;
    expect(pair[1].facts).toEqual(pair[0].facts);
    expect(pair[0].facts.starts).toBe(1);
    expect(pair[0].hash).toBe(pair[1].hash);
    const later = await invoke();
    expect(later.facts.pid).toBe(hostPid);
    const cold = await invoke(true);
    expect(cold.facts.pid).not.toBe(hostPid);
    expect(cold.hash).toBe(later.hash);
  } finally {
    if (hostPid) Bun.spawnSync(["kill", "-TERM", String(hostPid)], { stdout: "pipe", stderr: "pipe" });
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);
