// runPlaytest (`@origingame/engine/playtest`) inside a playtest file's
// `bun test` (playtest-files.ts): the script's commands through fresh runs
// in the worktree's session when available (dedicated processes otherwise),
// then the run's files read back as a PlaytestRun.
//
// Its settings come from the environment `og test` (playtest-run.ts) sets:
// OG_PLAYTEST_ENTRY (the game bundle), OG_PLAYTEST_ROOT (the project),
// OG_PLAYTEST_OUT (the file's output directory; each call writes
// `run-<n>/` there), OG_PLAYTEST_RESULT (a JSONL file: each run's record
// and each frame left unchecked, which playtest-run.ts sums) and
// OG_PLAYTEST_SESSION_SOCKET / _BUILD, and OG_PLAYTEST_ENV (og test's own
// environment, the runs' own: not bun test's NODE_ENV nor these settings).
// With OG_PLAYTEST_COLLECT a call records its PlaytestDesc there and plays
// nothing.
//
// Simulation-only playtests run without a renderer. Requested frames
// without a GPU are reported unchecked, never counted as passed.

import { expect } from "bun:test";
import { appendFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { FrameUnchecked, PLAYTEST_CLOCK, type PlaytestAssertions, type PlaytestDesc, type PlaytestLockstep, type PlaytestRun } from "../playtest.ts";
import { loadNativeHost, native } from "./index.ts";
import { decodePng } from "./png.ts";
import { runInSession } from "./session-client.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined>; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o?: { env?: Record<string, string | undefined>; stdout?: string; stderr?: string }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

const here = dirname(fileURLToPath(import.meta.url));
let calls = 0;

/** One line of OG_PLAYTEST_RESULT. */
function report(line: Record<string, unknown>): void {
  const file = process.env.OG_PLAYTEST_RESULT;
  if (file) appendFileSync(file, JSON.stringify(line) + "\n");
}

type ExpectResult = { tick: number; expect: string; kind: string; status: "pass" | "fail" | "unchecked"; expected: unknown; observed: unknown; reason: string };
type Summary = {
  tick: number;
  hash: string;
  netHash?: string;
  lockstep?: PlaytestLockstep;
  entities: number;
  quit: boolean;
  tier: string;
  renderer: string;
  adapterError: string | null;
  outputs: string[];
  warnings: string[];
  unchecked?: string[];
  expectations?: { failed: number; results: ExpectResult[] };
  assertions?: PlaytestAssertions;
};
const EMPTY_ASSERTIONS: PlaytestAssertions = { violations: [], uncovered: [], sites: [], defaults: { fields: 0, ticks: 0, values: 0, totalMs: 0, meanUs: 0, maxUs: 0 } };

/** The desc's own errors, before anything runs. */
function refusal(desc: PlaytestDesc): string | null {
  if (!desc || typeof desc.commands !== "string" || desc.commands.trim() === "") return "runPlaytest({ commands }) needs the script: `t:action[:args];...`, ending with quit";
  for (const i of desc.invariants ?? []) {
    if (/^(region|frame):/.test(i)) return `invariant '${i}' reads a frame, which every tick cannot afford: check it with an expect: step at the tick it matters`;
    if (i.includes(";")) return `invariant '${i}' holds a ';': one expectation each`;
  }
  const room = desc.room;
  if (room) {
    if (!(Math.floor(room.players) >= 2 && Math.floor(room.players) <= 16)) return `room.players is ${room.players}: a room playtest takes 2 to 16 processes`;
    if ((desc.runs ?? 1) > 1) return "a room playtest takes no `runs`: its processes play at once (a relaunch is a `then` step)";
    if (room.ticks !== undefined && !(Number.isInteger(room.ticks) && room.ticks > 0)) return `room.ticks is ${room.ticks}: the tick every process stops at, a whole number above 0 (or leave it out and end the script with quit)`;
    const name = room.name ?? "playtest";
    if (!/^[A-Za-z0-9._-]{1,54}$/.test(name)) return `room.name '${name}' is not a room id: 1 to 54 of letters, digits, '.', '_' and '-' (a then step's room adds .then-<n>)`;
  }
  return null;
}

export async function runPlaytest(desc: PlaytestDesc): Promise<PlaytestRun> {
  const refused = refusal(desc);
  if (refused) throw new Error(refused);
  const collect = process.env.OG_PLAYTEST_COLLECT;
  if (collect) {
    appendFileSync(collect, JSON.stringify(desc) + "\n");
    throw new Error("runPlaytest collected the script (OG_PLAYTEST_COLLECT): nothing was played");
  }
  const entry = process.env.OG_PLAYTEST_ENTRY;
  if (!entry) throw new Error("runPlaytest plays the game `og test` built: run a playtest file with `og test`, not a bare `bun test`");
  const root = process.env.OG_PLAYTEST_ROOT || undefined;
  const env = process.env.OG_PLAYTEST_ENV ? (JSON.parse(process.env.OG_PLAYTEST_ENV) as Record<string, string | undefined>) : process.env;
  const out = resolve(process.env.OG_PLAYTEST_OUT ?? "og-out/test", `run-${++calls}`);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });

  const failures: string[] = [];
  // The script's own checks: its expect: steps, its invariant sites and its require names.
  let checks = 0;
  // Region or frame expectations read with a renderer: what a run without one leaves out.
  let frameChecks = 0;
  // Every screenshot or frame expectation without a renderer, by the file it would have written.
  const unchecked: string[] = [];
  const adapterErrors = new Set<string>();
  const assertionLog: { violations: unknown[]; uncovered: unknown[]; required: Array<{ name: string; fired: boolean }>; defaults: unknown[] } = { violations: [], uncovered: [], required: [], defaults: [] };
  const captures = new Set<string>();

  // The host's clock is pinned (PlaytestDesc.clock), so og.platform.now is
  // reproducible; `store: "keep"` shares one local store file between the
  // call's processes, kept under a dot name, which no step's output may take
  // (an output named `platform.json` once replaced it: hollowbell FINDINGS
  // #18). In a room each place has its own (`platform-<place>.json`).
  const storeOf = (place: number): string | undefined => {
    if (desc.store !== "keep") return undefined;
    const path = join(out, ".store", desc.room && place > 0 ? `platform-${place}.json` : "platform.json");
    mkdirSync(dirname(path), { recursive: true });
    return path;
  };
  const fakeFile = desc.platform
    ? (() => {
        const path = join(out, ".store", "fake.json");
        mkdirSync(dirname(path), { recursive: true });
        writeFileSync(path, JSON.stringify(desc.platform));
        return path;
      })()
    : undefined;
  const invariantSteps = (desc.invariants ?? []).map((i) => `0:invariant:${i}`);
  const command = (commands: string, dir: string, clock: number, room?: { url: string; ticks?: number | undefined; place: number }) => {
    const script = [...invariantSteps, commands].join(";");
    const capture = join(dir, ".failure");
    captures.add(capture);
    const c = ["bun", join(here, "run.ts"), "--entry", entry, "--commands", script, "--out", dir, "--json", "--clock", String(clock), "--failure-capture", capture];
    if (room) c.push("--room", room.url);
    if (room?.ticks !== undefined) c.push("--ticks", String(room.ticks));
    if (root) c.push("--root", root);
    if (desc.seed !== undefined) c.push("--seed", String(desc.seed));
    if (desc.size) c.push("--size", desc.size);
    for (const step of (desc.setup ?? "").split(";").map((s) => s.trim()).filter((s) => s !== "")) c.push("--setup", step);
    const store = storeOf(room?.place ?? 0);
    if (store) c.push("--platform-store", store);
    if (fakeFile) c.push("--platform-fake", fakeFile);
    return c;
  };
  const summarise = (exitCode: number, stdout: string, stderr: string, what: string): Summary | null => {
    let summary: Summary | null = null;
    try {
      summary = JSON.parse(stdout.trim().split("\n").pop() ?? "");
    } catch {
      summary = null;
    }
    // A run whose only failures are its expectations or assertions exits 1 with its summary: each is a check.
    const expectedOnly = exitCode === 1 && summary !== null && ((summary.expectations?.failed ?? 0) > 0 || (summary.assertions?.violations.length ?? 0) > 0);
    if ((exitCode !== 0 && !expectedOnly) || !summary) {
      failures.push(`${what} failed (exit ${exitCode}): ${stderr.trim() || stdout.trim()}`);
      return null;
    }
    if (summary.adapterError && !adapterErrors.has(summary.adapterError)) {
      adapterErrors.add(summary.adapterError);
      const expected = env.OG_RENDERER?.trim().toLowerCase() === "none";
      process.stderr.write(`WARNING ${expected ? "" : "unexpected "}missing GPU: ${summary.adapterError}; frame checks are unchecked, not passed\n`);
    }
    for (const e of summary.expectations?.results ?? []) {
      if (e.status === "unchecked") unchecked.push(`${what}: expect:${e.expect}`);
      else {
        checks++;
        if (e.kind === "region" || e.kind === "frame") frameChecks++;
        if (e.status === "fail") failures.push(`${what}: expect:${e.expect} failed at tick ${e.tick}: expected ${JSON.stringify(e.expected)}, observed ${JSON.stringify(e.observed)}; ${e.reason}`);
      }
    }
    const a = summary.assertions ?? EMPTY_ASSERTIONS;
    // Every invariant site (the playtest's, a failed default, the game's always) is one check per process.
    checks += a.sites.filter((s) => s.kind === "always" || s.kind === "unreachable").length;
    for (const v of a.violations) {
      const where = [v.entity && `on ${v.entity}`, v.component && v.field && `${v.component}.${v.field}`].filter(Boolean).join(" ");
      failures.push(`${what}: ${v.origin} ${v.kind} '${v.name}'${v.location && v.origin === "game" ? ` (${v.location})` : ""} failed at tick ${v.tick}${where ? ` ${where}` : ""}: ${v.message}; observed ${JSON.stringify(v.value)}${v.expected !== null && v.expected !== undefined ? `, expected ${JSON.stringify(v.expected)}` : ""}${v.details !== null && v.details !== undefined ? `, details ${JSON.stringify(v.details)}` : ""}${v.failures > 1 ? ` (${v.failures} failing calls)` : ""}`);
    }
    assertionLog.violations.push(...a.violations.map((v) => ({ run: what, ...v })));
    assertionLog.uncovered.push(...a.uncovered.map((u) => ({ run: what, ...u })));
    assertionLog.defaults.push({ run: what, ...a.defaults });
    return summary;
  };
  const spawn = async (c: string[], what: string): Promise<Summary | null> => {
    try {
      const shared = await runInSession(c.slice(2), process.env.OG_PLAYTEST_SESSION_SOCKET, process.env.OG_PLAYTEST_SESSION_BUILD, undefined, "run", env);
      if (shared) return summarise(shared.exitCode, shared.stdout, shared.stderr, what);
    } catch (error) {
      // A lost reply can follow persistent store writes. Never replay it alone.
      failures.push(`${what}: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
    const r = Bun.spawnSync(c, { env, stdout: "pipe", stderr: "pipe" });
    return summarise(r.exitCode, new TextDecoder().decode(r.stdout), new TextDecoder().decode(r.stderr), what);
  };
  /**
   * PlaytestDesc.require over every process of the playtest (the script, its `then` steps, the
   * room's peers): a named site (or `name[key]` bucket) fired when it fired in any of them, and an
   * each's buckets are those any of them saw, each held when it held in one.
   */
  const reports: PlaytestAssertions[] = [];
  const requireFired = (what: string): void => {
    type Merged = { kind: string; name: string; ran: boolean; fired: boolean; buckets: Map<string, boolean> };
    const merged = new Map<string, Merged>();
    for (const a of reports)
      for (const s of a.sites) {
        if (s.kind === "always" || s.kind === "unreachable") continue;
        const key = `${s.kind}\u0000${s.name}\u0000${s.location}`;
        const m = merged.get(key) ?? { kind: s.kind, name: s.name, ran: false, fired: false, buckets: new Map() };
        m.ran ||= s.ran;
        m.fired ||= s.fired;
        for (const b of s.buckets ?? []) m.buckets.set(b.key, (m.buckets.get(b.key) ?? false) || b.fired);
        merged.set(key, m);
      }
    for (const want of desc.require ?? []) {
      checks++;
      const m = /^(.*)\[(.*)\]$/.exec(want);
      const name = m ? m[1]! : want;
      const sites = [...merged.values()].filter((s) => s.name === name);
      let fired: boolean;
      let why = "";
      if (sites.length === 0) {
        fired = false;
        why = `no og.assert.sometimes, reachable or each named '${name}' ran or was declared: check the name`;
      } else if (m) {
        fired = sites.some((s) => s.buckets.get(m[2]!) === true);
        if (!fired) why = `bucket [${m[2]}] never held${sites.some((s) => s.buckets.has(m[2]!)) ? "" : " (never seen)"}`;
      } else {
        fired = sites.some((s) => s.fired && (s.kind !== "each" || [...s.buckets.values()].every(Boolean)));
        if (!fired) {
          const s = sites[0]!;
          const open = [...s.buckets].filter(([, held]) => !held).map(([key]) => key);
          why = s.kind === "each" && open.length > 0 ? `buckets never held: ${open.join(", ")}` : s.ran ? "it ran but never held" : "the run never got there";
        }
      }
      assertionLog.required.push({ name: want, fired });
      if (!fired) failures.push(`${what}: required ${sites[0]?.kind ?? "assertion"} '${want}' never fired: ${why}`);
    }
  };
  const record = (extra: Record<string, unknown>) => report({ kind: "run", out, checks, frameChecks, unchecked, adapterErrors: [...adapterErrors], captures: [...captures], assertions: assertionLog, failures, ...extra });
  const refuse = (extra: Record<string, unknown> = {}): never => {
    record({ ...extra, failed: true });
    throw new Error(`the playtest's run failed:\n- ${failures.join("\n- ")}\n(its files: ${out})`);
  };
  const clock = desc.clock ?? PLAYTEST_CLOCK;

  // PlaytestDesc.room: a relay in this process for this call, and every
  // peer's process at once, each joined to the room (the first is the run,
  // the others its peers). A `then` step relaunches every place at once into
  // a room of its own (`<name>.then-<n>`): lockstep keeps the steps on the
  // room's shared ticks, so a `quit` ends every place on one tick.
  const room = desc.room;
  const peerSummaries: Summary[] = [];
  let summary: Summary | null;
  let playRoom: ((name: string, scripts: (place: number) => string, dirOf: (place: number) => string, clock: number, what: string) => Promise<Summary[]>) | null = null;
  if (room) {
    const players = Math.floor(room.players);
    loadNativeHost();
    const relay = JSON.parse(native().og_node_relay_start(0)) as { ok: boolean; value?: number; reason?: string };
    if (!relay.ok) {
      failures.push(`the playtest's relay did not start: ${relay.reason}`);
      refuse();
    }
    playRoom = async (roomName, scripts, dirOf, at, what) => {
      const url = `ws://127.0.0.1:${relay.value}/${roomName}`;
      const results = await Promise.all(
        Array.from({ length: players }, async (_, i) => {
          const dir = dirOf(i);
          mkdirSync(dir, { recursive: true });
          const p = Bun.spawn(command(scripts(i), dir, at, { url, ticks: room.ticks, place: i }), { env, stdout: "pipe", stderr: "pipe" });
          const [code, stdout, stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
          return summarise(code, stdout, stderr, `${what} (peer ${i})`);
        }),
      );
      if (results.some((r) => r === null)) refuse();
      return results as Summary[];
    };
    const results = await playRoom(room.name ?? "playtest", (i) => room.commands?.[i] ?? desc.commands, (i) => (i === 0 ? out : join(out, `peer-${i}`)), clock, "og run");
    summary = results[0]!;
    peerSummaries.push(...results.slice(1));
    for (const [i, s] of peerSummaries.entries()) for (const f of s.unchecked ?? []) unchecked.push(`peer ${i + 1} ${f}`);
  } else summary = await spawn(command(desc.commands, out, clock), "og run");
  if (!summary) return refuse();
  for (const s of [summary, ...peerSummaries]) reports.push(s.assertions ?? EMPTY_ASSERTIONS);
  const rendered = summary.tier !== "none";
  unchecked.push(...(summary.unchecked ?? []));
  // More runs of the same script (desc.runs), each with a fresh world into
  // its own directory: only their end hashes are kept.
  const hashes = [summary.hash];
  for (let i = 1; i < Math.max(1, Math.floor(desc.runs ?? 1)); i++) {
    const again = await spawn(command(desc.commands, join(out, `.run-${i + 1}`), clock), `run ${i + 1} of ${desc.runs}`);
    if (again) hashes.push(again.hash);
  }

  // The frames the call's runs took without a renderer, by name: run.frame lists only the others.
  const missedNames = new Set<string>(summary.unchecked ?? []);
  const runOf = (s: Summary, dir: string, later: readonly PlaytestRun[], peers: readonly PlaytestRun[] = []): PlaytestRun => {
    const path = (f: string) => {
      const p = resolve(dir, f);
      if (!existsSync(p)) throw new Error(`the run wrote no ${f} (it wrote ${s.outputs.map((o) => o.slice(dir.length + 1)).join(", ") || "nothing"})`);
      return p;
    };
    const missed = new Set(s.unchecked ?? []);
    for (const f of missed) missedNames.add(f);
    const runRendered = s.tier !== "none";
    return {
      tick: s.tick,
      hash: s.hash,
      hashes: dir === out ? hashes : [s.hash],
      entities: s.entities,
      quit: s.quit,
      tier: s.tier,
      rendered: runRendered,
      unchecked: [...missed],
      files: s.outputs.map((o) => o.slice(dir.length + 1)),
      warnings: s.warnings,
      then: later,
      assertions: s.assertions ?? EMPTY_ASSERTIONS,
      netHash: s.netHash ?? "",
      lockstep: s.lockstep ?? null,
      peers,
      json: <T>(f: string) => JSON.parse(readFileSync(path(f), "utf8")) as T,
      text: (f: string) => readFileSync(path(f), "utf8"),
      png: (f: string) => {
        if (missed.has(f)) throw new FrameUnchecked(f);
        const bytes = readFileSync(path(f));
        const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        if (bytes.length < 24 || view.getUint32(12) !== 0x49484452) throw new Error(`${f} is not a PNG`);
        return { width: view.getUint32(16), height: view.getUint32(20), bytes: bytes.length };
      },
      pixels: (f: string) => {
        if (missed.has(f)) throw new FrameUnchecked(f);
        return decodePng(new Uint8Array(readFileSync(path(f))));
      },
      wav: (f: string) => {
        const bytes = readFileSync(path(f));
        const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        if (bytes.length < 44 || view.getUint32(0) !== 0x52494646 || view.getUint32(8) !== 0x57415645) throw new Error(`${f} is not a WAV`);
        const rate = view.getUint32(24, true);
        const channels = view.getUint16(22, true);
        const samples = (bytes.length - 44) >> 1;
        let peak = 0;
        let sum = 0;
        for (let i = 0; i < samples; i++) {
          const x = view.getInt16(44 + i * 2, true) / 32767;
          peak = Math.max(peak, Math.abs(x));
          sum += x * x;
        }
        return { rate, frames: samples / Math.max(1, channels), peak, rms: samples > 0 ? Math.sqrt(sum / samples) : 0 };
      },
      frame(files, check) {
        const names = typeof files === "string" ? [files] : [...files];
        if (runRendered) return check();
        // Each frame is listed once, from the runs' own notes; a name no
        // screenshot took is listed too, so nothing is skipped silently.
        const fresh = names.filter((f) => !missedNames.has(f));
        for (const f of fresh) missedNames.add(f);
        report({ kind: "unchecked", files: fresh });
        // The test's assertion here is that each frame is reported unchecked, never that it passed.
        expect(names.every((f) => missedNames.has(f)), `${names.join(", ")}: no renderer, frame unchecked`).toBe(true);
      },
    };
  };
  // The later processes (desc.then), in order, after the script and its runs;
  // in a room every place again, at once (their runs are each step's `peers`).
  const later: PlaytestRun[] = [];
  for (const [i, step] of (desc.then ?? []).entries()) {
    const dir = join(out, `then-${i + 1}`);
    mkdirSync(dir, { recursive: true });
    if (playRoom) {
      const peerDir = (p: number) => join(dir, `peer-${p}`);
      const all = await playRoom(`${room!.name ?? "playtest"}.then-${i + 1}`, () => step.commands, (p) => (p === 0 ? dir : peerDir(p)), step.clock ?? clock, `then[${i}]`);
      for (const [p, s] of all.entries()) for (const f of s.unchecked ?? []) unchecked.push(`then[${i}]${p > 0 ? ` peer ${p}` : ""} ${f}`);
      for (const s of all) reports.push(s.assertions ?? EMPTY_ASSERTIONS);
      later.push(runOf(all[0]!, dir, [], all.slice(1).map((s, p) => runOf(s, peerDir(p + 1), []))));
      continue;
    }
    const s = await spawn(command(step.commands, dir, step.clock ?? clock), `then[${i}]`);
    if (!s) return refuse();
    for (const f of s.unchecked ?? []) unchecked.push(`then[${i}] ${f}`);
    reports.push(s.assertions ?? EMPTY_ASSERTIONS);
    later.push(runOf(s, dir, []));
  }
  requireFired(desc.then?.length || room ? "the playtest's processes" : "og run");
  const run = runOf(
    summary,
    out,
    later,
    peerSummaries.map((s, i) => runOf(s, join(out, `peer-${i + 1}`), [])),
  );
  const extra = {
    tick: summary.tick,
    hash: summary.hash,
    hashes,
    tier: summary.tier,
    rendered,
    renderer: summary.renderer,
    adapterError: summary.adapterError,
    warnings: summary.warnings,
    ...(room ? { netHashes: [summary, ...peerSummaries].map((s) => s.netHash ?? ""), lockstep: [summary, ...peerSummaries].map((s) => s.lockstep ?? null) } : {}),
  };
  if (failures.length > 0) refuse(extra);
  record(extra);
  // The script's checks held (or are reported unchecked): that is one assertion of the test.
  if (checks > 0 || unchecked.length > 0) expect(failures, "the script's expect: steps, invariants and required sites").toEqual([]);
  return run;
}
