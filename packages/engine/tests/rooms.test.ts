// Rooms and lockstep on the native host (docs/design.md §2.8, §2.10): `og
// relay` and two `og run --room` processes. The lockstep input fixture, each
// process on its own scripted bot, hashes the same at tick 600, and a peer frozen past
// stallTimeoutMs is dropped at one tick, then takes its seat back from the
// room's world.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
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

// Wall time decides this one: a 2.5 s SIGSTOP against a 0.8 s stallTimeoutMs,
// both peers paced at 60 Hz. A loaded machine slips the pacing and the test
// times out, so it runs in the perf lane (scripts/perf.sh sets OG_PERF=1,
// under exclusive admission), not in the gate.
test.skipIf(!process.env.OG_PERF)("a peer frozen past stallTimeoutMs is dropped at one tick, then takes its seat back from the room's world", async () => {
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
