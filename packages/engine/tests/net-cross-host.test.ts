// A lockstep session between the native and the web host (docs/design.md
// §2.10, G2-7): one peer on the napi addon, the other on the wasm module,
// in one room of `og relay`, playing the rts-commands fixture (a game's own
// input shape, per-tick commands). Each peer is its own process
// (tests/support/net-peer.ts) carrying its room over a WebSocket, so both
// reach the relay the same way. They must start lockstep, see every order
// once, never desync and confirm one world, in lockstep and in rollback
// through latency, jitter and loss: the engine's simulation is the same
// bits on both hosts.
//
// The ways this could fail: a field encoded differently on the two hosts
// (the wire's packed inputs, a command record), a float the wasm rounds
// another way, a snapshot one host cannot restore (rollback's rewind), or
// og.Desync firing at the first hash. The node-and-node session is the
// control: it shows the harness itself plays one world.
//
// The web side needs a wasm module built from this tree, which only the
// gate's wasm build makes: with OG_WASM_STALE=1, without dist/web, or with
// a module older than this tree's ABI (its glue lacks a binding symbol
// SYMBOLS lists, which bindHost would refuse) the cross-host cases skip.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { SYMBOLS } from "../src/generated/abi.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): {
    exited: Promise<number>;
    stdout: ReadableStream<Uint8Array>;
    stderr: ReadableStream<Uint8Array>;
    kill(): void;
  };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const glue = join(pkg, "dist", "web", "origin-engine.js");
/** Why the web peer cannot run here, or null when the module is this tree's. */
async function webModule(): Promise<string | null> {
  if (process.env.OG_WASM_STALE === "1") return "OG_WASM_STALE=1";
  if (!existsSync(glue) || !existsSync(join(pkg, "dist", "web", "origin-engine_bg.wasm"))) return "no dist/web";
  // The glue's exports are the web host's ABI as is (src/web/host.ts): importing it starts no wasm.
  const exports = (await import(glue)) as Record<string, unknown>;
  const missing = SYMBOLS.filter((s) => typeof exports[s] !== "function");
  return missing.length > 0 ? `dist/web is stale: it lacks ${missing.length} binding symbol(s), ${missing.slice(0, 3).join(", ")}` : null;
}
const NO_WASM = await webModule();
if (NO_WASM) console.warn(`net-cross-host: the cross-host cases skip (${NO_WASM})`);

type Peer = {
  host: string;
  tick: number;
  confirmedHash: string | null;
  status: { running: boolean; peer: number; startTick: number | null; desync: number | null; rollbacks: number };
  facts: { orders0: number; orders1: number; lastTick: number };
};

async function session(hosts: [string, string], room: string, extra: string[] = [], shaping: string[] = []): Promise<Peer[]> {
  const relay = Bun.spawn([ogBin(), "relay", "--port", "0", ...shaping], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  try {
    const reader = relay.stdout.getReader();
    let text = "";
    while (!/:(\d+)\//.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og relay exited: ${text}`);
      text += new TextDecoder().decode(value);
    }
    const port = /:(\d+)\//.exec(text)![1]!;
    return await Promise.all(
      hosts.map(async (host) => {
        const p = Bun.spawn(["bun", "tests/support/net-peer.ts", "--host", host, "--port", port, "--room", room, "--ticks", "600", ...extra], {
          cwd: pkg,
          stdout: "pipe",
          stderr: "pipe",
        });
        const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
        if (code !== 0) throw new Error(`${host} peer exited ${code}: ${err}${out}`);
        return JSON.parse(out.trim().split("\n").pop()!) as Peer;
      }),
    );
  } finally {
    relay.kill();
  }
}

function expectOneWorld(peers: Peer[]): void {
  expect(peers.map((p) => p.status.peer).sort()).toEqual([0, 1]);
  for (const p of peers) {
    expect(p.status.running).toBe(true);
    expect(p.status.startTick!).toBeLessThan(58);
    expect(p.status.desync).toBeNull();
    // Orders at ticks 100 to 380 act inputDelayTicks (2) ticks on: the last at 382.
    expect(p.facts).toMatchObject({ orders0: 30, orders1: 30, lastTick: 382 });
    expect(p.confirmedHash).not.toBeNull();
  }
  expect(peers[1]!.confirmedHash).toBe(peers[0]!.confirmedHash);
}

test("the harness: two native peers confirm one world", async () => {
  expectOneWorld(await session(["node", "node"], "xh-control"));
}, 300_000);

test.skipIf(NO_WASM !== null)("a native and a web peer play one lockstep session and confirm one world", async () => {
  const peers = await session(["node", "web"], "xh-lockstep");
  expectOneWorld(peers);
  expect(peers.map((p) => p.host)).toEqual(["node", "web"]);
}, 300_000);

test.skipIf(NO_WASM !== null)("a native and a web peer roll back through 80 ms, jitter and loss and confirm one world", async () => {
  const peers = await session(["web", "node"], "xh-rollback", ["--rollback", "8"], ["--latency", "80", "--jitter", "30", "--loss", "5", "--seed", "3"]);
  expectOneWorld(peers);
  // Each host rewound its own snapshot and re-simulated: the rewind is exercised on both.
  for (const p of peers) expect(p.status.rollbacks).toBeGreaterThan(0);
}, 300_000);
