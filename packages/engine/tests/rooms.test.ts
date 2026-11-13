// Rooms and lockstep on the native host (docs/design.md §2.8, §2.10): `og
// relay` and two `og run --room` processes. The lockstep input fixture, each
// process on its own scripted bot, hashes the same at tick 600; a game whose
// system reads Math.random() is caught: og.Desync names the first divergent
// tick and `dump_desync` the component and field. A peer whose connection is
// cut in the middle of a run comes back by itself (og-net's client), takes
// its seat back from the room's world, and both end on one hash.
import { expect, test } from "bun:test";
import * as net from "node:net";
import { dirname, join } from "node:path";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd: string }): { exitCode: number };
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): {
    exited: Promise<number>;
    stdout: ReadableStream<Uint8Array>;
    stderr: ReadableStream<Uint8Array>;
    kill(signal?: string): void;
    pid: number;
  };
};

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

/** The part of node:net a TCP wire between a peer and the relay needs (src/env.d.ts declares only what the engine uses). */
type Tcp = { on(event: "data" | "error" | "close", listener: (data: Uint8Array) => void): Tcp; write(data: Uint8Array): void; destroy(): void };
const tcp = net as unknown as {
  createServer(listener: (socket: Tcp) => void): { listen(port: number, host: string, done: () => void): void; address(): { port: number }; close(): void };
  connect(port: number, host: string): Tcp;
};

/**
 * `og relay --port 0` (with `shaping`: --latency, --jitter, --loss, --seed), the port it printed, and `logged`, which
 * resolves once the relay has logged a line matching `line` (such as a peer joining a room).
 */
async function relay(shaping: string[] = []): Promise<{ port: number; stop: () => void; logged: (line: RegExp) => Promise<void> }> {
  const p = Bun.spawn([ogBin(), "relay", "--port", "0", ...shaping], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  const reader = p.stdout.getReader();
  let text = "";
  while (!/:(\d+)\//.test(text)) {
    const { value, done } = await reader.read();
    if (done) throw new Error(`og relay exited: ${text}${await new Response(p.stderr).text()}`);
    text += new TextDecoder().decode(value);
  }
  const logged = async (line: RegExp) => {
    while (!line.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og relay exited before logging ${line}: ${text}`);
      text += new TextDecoder().decode(value);
    }
  };
  return { port: Number(/:(\d+)\//.exec(text)![1]), stop: () => p.kill(), logged };
}

type Summary = {
  tick: number;
  netHash: string;
  confirmedHash: string | null;
  lockstep: { running: boolean; peer: number; startTick: number | null; desync: number | null; rollback: boolean; rollbacks: number; resimulatedTicks: number; finalTick: number | null; drops: number };
  facts: { droppedAt: number; drops: number; rejoinedAt: number; rejoins: number } & Record<string, unknown> | null;
};

/**
 * Two `og run --room` processes at once, each with its own extra arguments; `during` gets each process once they started.
 * `paced` leaves out --headless, so each run takes its ticks at the game's rate on the wall clock (og run's pacing)
 * instead of as fast as it can: a test that acts on wall time then lands on the same ticks on any machine.
 */
async function pair(
  project: string,
  room: string,
  args: string[][],
  shaping: string[] = [],
  during?: (pids: number[]) => Promise<void>,
  paced = false,
): Promise<Summary[]> {
  const r = await relay(shaping);
  const processes: number[] = [];
  try {
    const runs = Promise.all(
      args.map(async (extra) => {
        const p = Bun.spawn([ogBin(), "run", project, ...(paced ? [] : ["--headless"]), "--no-render", "--json", "--room", `ws://127.0.0.1:${r.port}/${room}`, ...extra], {
          cwd: repo,
          stdout: "pipe",
          stderr: "pipe",
        });
        processes.push(p.pid);
        const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
        if (code !== 0) throw new Error(`og run ${extra.join(" ")} exited ${code}: ${err}${out}`);
        return JSON.parse(out.trim().split("\n").pop()!) as Summary;
      }),
    );
    if (during) await during(processes);
    return await runs;
  } finally {
    r.stop();
  }
}

test("two runs through the relay hash the same at tick 600", async () => {
  const [a, b] = await pair("tests/fixtures/lockstep-room", "duel", [
    ["--ticks", "600", "--setup", "bot:1"],
    ["--ticks", "600", "--setup", "bot:2"],
  ]);
  expect(a!.tick).toBe(600);
  expect(b!.tick).toBe(600);
  expect(a!.lockstep.running && b!.lockstep.running).toBe(true);
  expect([a!.lockstep.peer, b!.lockstep.peer].sort()).toEqual([0, 1]);
  expect(a!.lockstep.desync).toBeNull();
  expect(b!.lockstep.desync).toBeNull();
  expect(b!.netHash).toBe(a!.netHash);
}, 300_000);

test("Math.random() in a system is a desync: og.Desync names tick 101 and dump_desync the field", async () => {
  const outs = [mkdtempSync(join(tmpdir(), "og-desync-")), mkdtempSync(join(tmpdir(), "og-desync-"))];
  try {
    const script = "4.5:get:Seen:seen.json;4.6:dump_desync:desync.json";
    const runs = await pair(
      "packages/engine/tests/fixtures/desync",
      "desync",
      outs.map((out) => ["--ticks", "300", "--out", out, "--commands", script]),
    );
    for (const [i, run] of runs.entries()) {
      // The system ran at tick 100: the worlds after it (og.Time.tick 101) differ.
      expect(run.lockstep.desync).toBe(101);
      const seen = JSON.parse(readFileSync(join(outs[i]!, "seen.json"), "utf8")).value as { tick: number; peer: number };
      expect(seen).toEqual({ tick: 101, peer: 1 - run.lockstep.peer });
      const dump = JSON.parse(readFileSync(join(outs[i]!, "desync.json"), "utf8"));
      expect(dump.tick).toBe(101);
      expect(dump.local.hash).not.toBe(dump.remote.hash);
      expect(dump.diff.resources).toEqual([]);
      expect(dump.diff.components.length).toBe(1);
      expect(dump.diff.components[0].component).toBe("Wobble");
      expect(dump.diff.components[0].entity).toBe("wobbler");
      expect(dump.diff.components[0].fields.map((f: { field: string }) => f.field)).toEqual(["offset"]);
    }
  } finally {
    for (const o of outs) rmSync(o, { recursive: true, force: true });
  }
}, 300_000);

test("two peers whose platforms answer one purchase differently keep one hash: a shared-state system hears neither", async () => {
  const outs = [mkdtempSync(join(tmpdir(), "og-session-")), mkdtempSync(join(tmpdir(), "og-session-"))];
  try {
    const fakes = ["ok", "cancelled"].map((outcome, i) => {
      const file = join(outs[i]!, "fake.json");
      writeFileSync(file, JSON.stringify({ outcomes: { "purchase.buy": [outcome] } }));
      return file;
    });
    const script = "4:heard;4.2:get:Wallet:wallet.json;4.2:get:Heard:heard.json";
    const runs = await pair(
      "packages/engine/tests/fixtures/platform-session",
      "session",
      outs.map((out, i) => ["--ticks", "300", "--out", out, "--commands", script, "--platform-fake", fakes[i]!]),
    );
    for (const [i, run] of runs.entries()) {
      expect(run.lockstep.running).toBe(true);
      expect(run.lockstep.desync).toBeNull();
      // Credit writes the shared Wallet: it heard no answer and read no result.
      expect(JSON.parse(readFileSync(join(outs[i]!, "wallet.json"), "utf8")).value).toEqual({ gems: 0, answers: 0 });
      // Toast writes nothing shared: it heard its peer's own answer.
      expect(JSON.parse(readFileSync(join(outs[i]!, "heard.json"), "utf8")).value).toEqual({ count: 1 });
    }
    expect(runs[1]!.netHash).toBe(runs[0]!.netHash);
  } finally {
    for (const o of outs) rmSync(o, { recursive: true, force: true });
  }
}, 300_000);

test("a room playtest's quit and its then step land on one tick for every place", async () => {
  // playtest-run.ts, as `og test` runs it, on the room-then playtest over the input fixture.
  // Before: a room playtest refused `then`, and a joiner that took the authority's snapshot
  // moved its steps with the jump, so its `quit` landed ticks after the authority's.
  const { bundleGame } = await import("../src/node/build.ts");
  const out = mkdtempSync(join(tmpdir(), "og-room-then-"));
  try {
    const entry = join(out, "game.js");
    const arena = join(repo, "tests/fixtures/lockstep-room");
    expect(Bun.spawnSync([ogBin(), "cook", arena], { cwd: repo }).exitCode).toBe(0);
    const b = await bundleGame(join(arena, "src/main.ts"), entry, "native");
    expect(b.ok).toBe(true);
    const p = Bun.spawn(
      ["bun", join(repo, "packages/engine/src/node/playtest-run.ts"), "--file", join(repo, "packages/engine/tests/fixtures/room-then/then.og.ts"), "--entry", entry, "--root", arena, "--out", join(out, "test"), "--json"],
      { cwd: repo, stdout: "pipe", stderr: "pipe" },
    );
    const [code, stdout, stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
    const line = JSON.parse(stdout.trim().split("\n").pop() || "null") as { ok: boolean; checks: number; failures: string[]; netHashes: string[] } | null;
    expect(line?.failures ?? [stderr]).toEqual([]);
    expect(line!.ok).toBe(true);
    expect(code).toBe(0);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);

// G2-7: rollback over a bad network ends on the world lockstep makes from
// the same inputs. The bots read the tick alone (`bot:<n>:tick`), so both
// sessions get the same inputs; through 80 ms of latency, 30 of jitter and
// 5 % of input messages lost, the rollback peers predict, rewind and run
// ticks again, and the final world of tick 600 must hash as lockstep's.
test("rollback through latency, jitter and loss confirms tick 600 as lockstep does from the same inputs", async () => {
  const bots = [
    ["--ticks", "600", "--setup", "bot:1:tick"],
    ["--ticks", "600", "--setup", "bot:2:tick"],
  ];
  const lock = await pair("tests/fixtures/lockstep-room", "duel-lockstep", bots);
  // The bots hold still for 60 ticks: any start before tick 58 plays the same inputs.
  for (const r of lock) expect(r.lockstep.startTick!).toBeLessThan(58);
  expect(lock[0]!.tick).toBe(600);
  // A frame may take two ticks, so a run ends at 600 or 601: the final
  // world of tick 600 itself (og.net.finalHash) is what the peers compare.
  expect(lock[0]!.confirmedHash).not.toBeNull();
  expect(lock[1]!.confirmedHash).toBe(lock[0]!.confirmedHash);
  expect(lock[0]!.lockstep.rollbacks).toBe(0);

  const roll = await pair(
    "tests/fixtures/lockstep-room",
    "duel-rollback",
    bots.map((b) => [...b, "--setup", "rollback:8"]),
    ["--latency", "80", "--jitter", "30", "--loss", "5", "--seed", "7"],
  );
  for (const r of roll) {
    expect(r.lockstep.startTick!).toBeLessThan(58);
    expect(r.lockstep.rollback).toBe(true);
    expect(r.lockstep.desync).toBeNull();
    expect(r.lockstep.finalTick!).toBeGreaterThanOrEqual(600);
    expect(r.confirmedHash).toBe(lock[0]!.confirmedHash);
  }
  // It did roll back: predictions went wrong and ticks ran again.
  expect(roll[0]!.lockstep.rollbacks + roll[1]!.lockstep.rollbacks).toBeGreaterThan(10);
  expect(roll[0]!.lockstep.resimulatedTicks + roll[1]!.lockstep.resimulatedTicks).toBeGreaterThan(roll[0]!.lockstep.rollbacks);
}, 300_000);

// G2-7 batch 4: a game's own input shape (og.net.shape) over a room. Each
// seat's input is the fixture's Cursor, and its orders are commands: 30 per
// seat, given by the tick alone. Every order acts once on every peer, by the
// seat that gave it, in lockstep and in rollback through a lossy relay (a
// prediction repeats a seat's cursor, never its orders), and both sessions
// confirm one world.
test("an RTS's orders cross the room as commands once each, in lockstep and in rollback", async () => {
  const project = "packages/engine/tests/fixtures/rts-commands";
  const lock = await pair(project, "rts-lockstep", [["--ticks", "600"], ["--ticks", "600"]]);
  const roll = await pair(
    project,
    "rts-rollback",
    [["--ticks", "600", "--setup", "rollback:8"], ["--ticks", "600", "--setup", "rollback:8"]],
    ["--latency", "80", "--jitter", "30", "--loss", "5", "--seed", "3"],
  );
  for (const r of [...lock, ...roll]) {
    expect(r.lockstep.startTick!).toBeLessThan(58);
    expect(r.lockstep.desync).toBeNull();
    // Orders at ticks 100 to 380 act inputDelay (2) ticks on: the last at 382.
    expect(r.facts).toMatchObject({ orders0: 30, orders1: 30, lastTick: 382 });
    expect(r.confirmedHash).toBe(lock[0]!.confirmedHash);
  }
  expect(lock[0]!.confirmedHash).not.toBeNull();
  expect(roll[0]!.lockstep.rollbacks + roll[1]!.lockstep.rollbacks).toBeGreaterThan(5);
}, 300_000);

// G2-7 batch 5: quick play through `og relay`'s queue. Three engines connect
// to the relay without a room; two ask for a duel and are matched in a room
// of their own, where lockstep runs and both end on one hash; the third asks
// for another mode, matches nobody and hears `timeout`, in no room.
test("quick play matches two engines of one mode in a room of their own; a third of another mode times out", async () => {
  const runs = await pair("tests/fixtures/lockstep-room", "", [
    ["--ticks", "600", "--setup", "bot:1:tick", "--setup", "quick:duel"],
    ["--ticks", "600", "--setup", "bot:2:tick", "--setup", "quick:duel"],
    ["--ticks", "600", "--setup", "quick:other:1"],
  ]);
  const [a, b, c] = runs as (Summary & { room: { state: string; room: string; peers: unknown[] } })[];
  for (const r of [a!, b!]) {
    expect(r.room.state).toBe("in");
    expect(r.room.room).toMatch(/^quick-duel-\d+$/);
    expect(r.room.peers.length).toBe(2);
    expect(r.lockstep.running).toBe(true);
    expect(r.lockstep.desync).toBeNull();
  }
  expect(b!.room.room).toBe(a!.room.room);
  expect([a!.lockstep.peer, b!.lockstep.peer].sort()).toEqual([0, 1]);
  expect(b!.netHash).toBe(a!.netHash);
  expect(c!.room.state).toBe("none");
  expect(c!.lockstep.running).toBe(false);
}, 300_000);

test("a peer whose connection is cut comes back by itself, takes its seat back from the room's world, and both end on one hash", async () => {
  const r = await relay();
  // The second peer reaches the relay through a wire the test cuts once, 200 frames in (near tick 200 of 600).
  let frames = 0;
  let cuts = 0;
  const wire = tcp.createServer((down) => {
    const up = tcp.connect(r.port, "127.0.0.1");
    const end = () => (down.destroy(), up.destroy());
    down.on("data", (d) => {
      up.write(d);
      if (cuts === 0 && ++frames === 200) {
        cuts = 1;
        end();
      }
    });
    up.on("data", (d) => down.write(d));
    for (const s of [down, up]) s.on("error", end).on("close", end);
  });
  await new Promise<void>((done) => wire.listen(0, "127.0.0.1", done));
  try {
    // Paced (no --headless): the peer left alone must still be running when the other is back, a quarter of a second later.
    const run = async (port: number, bot: string) => {
      const p = Bun.spawn([ogBin(), "run", "tests/fixtures/lockstep-room", "--no-render", "--json", "--room", `ws://127.0.0.1:${port}/mend`, "--ticks", "600", "--setup", bot], { cwd: repo, stdout: "pipe", stderr: "pipe" });
      const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
      if (code !== 0) throw new Error(`og run exited ${code}: ${err}${out}`);
      return JSON.parse(out.trim().split("\n").pop()!) as Summary & { lockstep: { seats: { seat: number; peer: number; until: number | null }[] } };
    };
    // The second starts once the first is in the room, so the first is peer 0 however slowly it started.
    const first = run(r.port, "bot:1:tick");
    await r.logged(/room mend: peer 0 joined/);
    const [a, b] = await Promise.all([first, run(wire.address().port, "bot:2:tick")]);
    expect(cuts).toBe(1);
    // The room gave it a new number; the seat it held is its own again.
    expect([a.lockstep.peer, b.lockstep.peer]).toEqual([0, 2]);
    for (const s of [a, b]) {
      expect(s.tick).toBe(600);
      expect(s.lockstep.running).toBe(true);
      expect(s.lockstep.desync).toBeNull();
      expect(s.lockstep.seats.filter((h) => h.until === null).map((h) => [h.seat, h.peer])).toEqual([[0, 0], [1, 2]]);
      expect(s.facts!.drops).toBe(1);
      expect(s.facts!.rejoins).toBe(1);
    }
    expect(b.facts!.droppedAt).toBe(a.facts!.droppedAt);
    expect(b.facts!.rejoinedAt).toBe(a.facts!.rejoinedAt);
    expect(a.confirmedHash).not.toBeNull();
    expect(b.confirmedHash).toBe(a.confirmedHash);
  } finally {
    wire.close();
    r.stop();
  }
}, 120_000);

// Wall time decides this one: a 2.5 s SIGSTOP against a 0.8 s stallTimeout,
// both peers paced at 60 Hz. A loaded machine slips the pacing and the test
// times out, so it runs in the perf lane (scripts/perf.sh sets OG_PERF=1,
// under exclusive admission), not in the gate.
test.skipIf(!process.env.OG_PERF)("a peer frozen past stallTimeout is dropped at one tick, then takes its seat back from the room's world", async () => {
  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
  // Paced at 60 Hz, whatever the machine's speed: the stall from 2 s to 4.5 s, the drop near
  // tick 170 and the rejoin near tick 280 fall well inside 900 ticks (15 s).
  const common = ["--ticks", "900", "--setup", "stall:800"];
  const [a, b] = await pair(
    "tests/fixtures/lockstep-room",
    "stall",
    [
      [...common, "--setup", "bot:1:tick"],
      [...common, "--setup", "bot:2:tick"],
    ],
    [],
    async ([, second]) => {
      // A real stall: the engine's process (og run's bun child) stops, as a
      // frozen tab does, sends nothing, then runs again.
      const signal = (sig: string) => Bun.spawnSync(["pkill", `-${sig}`, "-P", String(second)]);
      await sleep(2000);
      signal("STOP");
      await sleep(2500);
      signal("CONT");
    },
    true,
  );
  for (const s of [a!, b!]) {
    expect(s.lockstep.running).toBe(true);
    expect(s.lockstep.desync).toBeNull();
    // og.SeatDropped and og.SeatRejoined are in the world: the rejoined peer has them from the room's.
    expect(s.facts!.drops).toBe(1);
    expect(s.facts!.rejoins).toBe(1);
  }
  expect(b!.facts!.droppedAt).toBe(a!.facts!.droppedAt);
  expect(b!.facts!.rejoinedAt).toBe(a!.facts!.rejoinedAt);
  // 2.5 s of silence against a 0.8 s timeout: dropped well before it came back.
  expect(a!.facts!.rejoinedAt - a!.facts!.droppedAt).toBeGreaterThan(60);
  expect(a!.confirmedHash).not.toBeNull();
  expect(b!.confirmedHash).toBe(a!.confirmedHash);
}, 120_000);
