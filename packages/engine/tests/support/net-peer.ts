// One peer of tests/net-cross-host.test.ts: the rts-commands fixture (a
// game's own input shape and per-tick commands) on one host (node: the napi
// addon; web: the wasm module outside a browser), in a room of an `og
// relay` the test started. The room is carried here the same way for both
// hosts, as the web shell carries a Player's: the engine's ops
// (og_room_take) go over a WebSocket in og-net's protocol and the relay's
// events come back through og_room_deliver, the room named as a Play link
// names it (og_room_set_host's `join`). Prints the session's end as JSON.
//
//   bun tests/support/net-peer.ts --host node|web --port <relay port> --room <id> --ticks <n> [--rollback <maxPrediction>]

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => (argv.includes(name) ? (argv[argv.indexOf(name) + 1] ?? "") : "");
const host = opt("--host");
const ticks = Number(opt("--ticks"));
const rollback = opt("--rollback");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}
const { boundHost, createEngine } = await import("../../src/runtime.ts");
const game = ((await import("../fixtures/rts-commands/src/main.ts")) as { default: GameToken }).default;

const created = createEngine(game);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const abi = boundHost()!.abi;
const must = (r: { ok: boolean; reason?: string }, what: string) => {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
};
must(engine.advance(0), "start");
if (rollback) must(engine.command("rollback", [rollback]), "rollback");

const socket = new WebSocket(`ws://127.0.0.1:${opt("--port")}/`);
await new Promise<void>((r, f) => {
  socket.addEventListener("open", () => r());
  socket.addEventListener("error", () => f(new Error("cannot reach the relay")));
});
const events: string[] = [];
socket.addEventListener("message", (m) => events.push(String(m.data)));
const pump = () => {
  for (const op of JSON.parse(abi.og_room_take(engine.id)) as unknown[]) socket.send(JSON.stringify(op));
  for (const e of events.splice(0)) abi.og_room_deliver(engine.id, e);
  // What the events made lockstep send goes now, not a loop later.
  for (const op of JSON.parse(abi.og_room_take(engine.id)) as unknown[]) socket.send(JSON.stringify(op));
};
abi.og_room_set_host(engine.id, JSON.stringify({ host: "player", join: opt("--room") }));

type Status = { running: boolean; peer: number; startTick: number | null; desync: number | null; acknowledged: number | null; rollbacks: number };
const status = () => JSON.parse(abi.og_net_status(engine.id)) as Status;
const finalHash = (t: number) => JSON.parse(abi.og_net_final_hash(engine.id, t)) as string | null;
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
let tick = 0;
for (let i = 0; ; i++) {
  if (i > 400_000) throw new Error(`stuck at tick ${tick}`);
  pump();
  const before = tick;
  const taken = engine.advance(1);
  must(taken, `tick ${tick}`);
  tick = (taken as { value: number }).value;
  const s = status();
  // In rollback the world reaches --ticks on predicted inputs: it goes on until that tick is final here and every peer has this one's inputs for it.
  if (tick >= ticks && finalHash(ticks) !== null && (s.acknowledged ?? ticks) >= ticks - 1) break;
  if (tick > ticks + 600) throw new Error(`tick ${ticks} never became final (at ${tick})`);
  // A held tick waits for the other peer's input; the lobby keeps the game's pace.
  if (tick === before) await sleep(1);
  else if (!s.running) await sleep(16);
}
// The other peer may still be waiting on an acknowledgement this one owes.
for (let i = 0; i < 50; i++) {
  pump();
  await sleep(10);
}
const facts = (engine.probe.state() as { facts?: unknown }).facts ?? null;
process.stdout.write(JSON.stringify({ host, tick, confirmedHash: finalHash(ticks), status: status(), facts }) + "\n");
socket.close();
process.exit(0);
