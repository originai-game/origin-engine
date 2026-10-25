// The Phase 2 core round's cases on the host named by --host (node: the
// napi addon, a dev build; web: the wasm module outside a browser, a release
// build; publish: the publish wasm), one --case per process. Prints JSON.
//
//   bun tests/support/core-round.ts --host node|web|publish --case <name>
//
// Cases:
//   rng         tests/fixtures/rng-readonly: a fork from og.Rng declared in
//               reads only, and one declared in writes, on tick 1
//   undeclared  one game per kind of undeclared access (a resource read, a
//               resource write, an og.Rng draw from reads, an event read, an
//               emit), each advanced 5 ticks: { [kind]: { ok, reason, log } }
//   time        og.platform.now and the answers' time under a fixed clock
//               (og run --clock) and under the system clock
//   input       input events' `at` within the tick: from the probe's input
//               (the host's stamp, clamped) and from the command channel's
//               step times; every KeyDown, KeyUp and PointerDown seen
//   latency     og.audio.outputLatency: 0 headless, then (web) what the
//               Worker reports for an AudioContext's latency

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineResource, defineSystem, defineComponent, f64, u32 } from "@origingame/engine";
import { createEngine, fixedClock, type Engine, type GameToken } from "../../src/runtime.ts";
import { localPlatform, memoryPersistence } from "../../src/platform-local.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => argv[argv.indexOf(name) + 1] ?? "";
const host = opt("--host");
const which = opt("--case");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const dir = host === "publish" ? "web-publish" : "web";
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", dir, "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", dir, "origin-engine_bg.wasm")));
}

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const out: Record<string, unknown> = { host, case: which };

if (which === "rng") {
  const { default: game, Repro } = await import("../fixtures/rng-readonly/src/main.ts");
  const engine = start(game);
  const advanced = engine.advance(3);
  const r = engine.world.resource(Repro);
  const rng = engine.world.resource(og.Rng);
  Object.assign(out, {
    advanced,
    repro: r.ok ? { ...r.value } : r.reason,
    rng: rng.ok ? { seed: rng.value.seed, state: rng.value.state } : rng.reason,
    hash: host === "publish" ? "" : engine.hash(),
  });
} else if (which === "undeclared") {
  const Shared = defineResource("CrShared", { n: u32(7) });
  const Seen = defineResource("CrSeen", { tries: u32(), refused: u32(), draw: f64() });
  const Ping = defineComponent("CrPing", { n: u32() });
  void Ping;
  const kinds = {
    read: defineSystem("CrRead", { writes: [Seen] }, (_q, _t, world) => {
      const s = world.resource(Seen);
      if (!s.ok) return;
      s.value.tries += 1;
      if (!world.resource(Shared).ok) s.value.refused += 1;
    }),
    write: defineSystem("CrWrite", { reads: [Shared], writes: [Seen] }, (_q, _t, world) => {
      const s = world.resource(Seen);
      const shared = world.resource(Shared);
      if (!s.ok || !shared.ok) return;
      s.value.tries += 1;
      shared.value.n = 8;
    }),
    draw: defineSystem("CrDraw", { reads: [og.Rng], writes: [Seen] }, (_q, _t, world) => {
      const s = world.resource(Seen);
      const rng = world.resource(og.Rng);
      if (!s.ok || !rng.ok) return;
      s.value.tries += 1;
      const d = rng.value.next();
      if (Number.isNaN(d)) s.value.refused += 1;
    }),
    events: defineSystem("CrEvents", { writes: [Seen] }, (_q, _t, world) => {
      const s = world.resource(Seen);
      if (!s.ok) return;
      s.value.tries += 1;
      void world.events(og.KeyDown);
    }),
    emit: defineSystem("CrEmit", { writes: [Seen] }, (_q, _t, world) => {
      const s = world.resource(Seen);
      if (!s.ok) return;
      s.value.tries += 1;
      if (!world.emit(og.KeyUp, { key: "KeyA", at: 0 }).ok) s.value.refused += 1;
    }),
  };
  for (const [kind, system] of Object.entries(kinds)) {
    const engine = start(og.game({ resources: [Shared, Seen], systems: [system] }));
    const advanced = engine.advance(5);
    const seen = engine.world.resource(Seen);
    const shared = engine.world.resource(Shared);
    out[kind] = {
      ok: advanced.ok,
      reason: advanced.ok ? "" : advanced.reason.split("\n")[0],
      tick: engine.world.tick,
      seen: seen.ok ? { tries: seen.value.tries, refused: seen.value.refused } : null,
      shared: shared.ok ? shared.value.n : null,
      log: engine.takeLog().filter((l) => l.warning).map((l) => l.text),
    };
    engine.destroy();
  }
} else if (which === "time") {
  const Clock = defineResource("CrClock", { startNow: f64(), request: u32(), answerTime: f64(), answerTick: u32(), nowThen: f64(), nowAt5: f64() });
  const Ask = defineSystem("CrAsk", { writes: [Clock] }, (_q, time, world) => {
    const c = world.resource(Clock);
    if (!c.ok) return;
    if (time.tick === 0) c.value.startNow = og.platform.now(world);
    if (time.tick === 1) {
      const asked = og.platform.storeGet(world, "save");
      c.value.request = asked.ok ? asked.value : 0;
    }
    const r = c.value.request === 0 ? null : og.platform.result(world, c.value.request);
    if (r !== null && c.value.answerTick === 0) {
      c.value.answerTime = r.time;
      c.value.answerTick = time.tick;
      c.value.nowThen = og.platform.now(world);
    }
    if (time.tick === 5) c.value.nowAt5 = og.platform.now(world);
  });
  const T = 1767225600000;
  const fixed = start(og.game({ tick: 60, resources: [Clock], systems: [Ask] }));
  fixed.setClock(fixedClock(fixed, T));
  fixed.attachPlatform(localPlatform(memoryPersistence()));
  for (let i = 0; i < 8; i++) fixed.advance(1);
  const c = fixed.world.resource(Clock);
  const info = og.platform.info(fixed.world);
  const system = start(og.game({ tick: 60, resources: [Clock], systems: [Ask] }));
  system.attachPlatform(localPlatform(memoryPersistence()));
  const before = Date.now();
  system.advance(1);
  const s = system.world.resource(Clock);
  Object.assign(out, {
    T,
    fixed: c.ok ? { ...c.value } : c.reason,
    clock: info.clock,
    nowAfter: og.platform.now(fixed.world),
    systemClock: og.platform.info(system.world).clock,
    systemSkewMs: s.ok ? Math.abs(s.value.startNow - before) : -1,
  });
} else if (which === "input") {
  const seen: Array<{ tick: number; kind: string; key?: string; id?: number; at: number }> = [];
  const Watch = defineSystem("CrWatch", { reads: [og.KeyDown, og.KeyUp, og.PointerDown] }, (_q, time, world) => {
    for (const e of world.events(og.KeyDown)) seen.push({ tick: time.tick, kind: "down", key: e.key, at: e.at });
    for (const e of world.events(og.KeyUp)) seen.push({ tick: time.tick, kind: "up", key: e.key, at: e.at });
    for (const e of world.events(og.PointerDown)) seen.push({ tick: time.tick, kind: "pointer", id: e.id, at: e.at });
  });
  const engine = start(og.game({ tick: 60, systems: [Watch] }));
  const p = engine.probe as unknown as { input(e: unknown): boolean; commands(s: string): unknown };
  p.input({ kind: "key", key: "KeyA", action: "down", at: 0.25 });
  engine.advance(1);
  p.input({ kind: "key", key: "KeyA", action: "up", at: -0.5 });
  p.input({ kind: "pointer", action: "down", x: 10, y: 20, button: "left", id: 2, at: 0.75 });
  engine.advance(1);
  p.input({ kind: "key", key: "KeyB", action: "press", at: 3 });
  p.input({ kind: "key", key: "KeyC", action: "down" });
  engine.advance(2);
  // Scheduled at tick 4: 0.1 s is 6 ticks on, a boundary (tick 10, at 0); 0.1083 s is 6.498 ticks,
  // which rounds to 6 (tick 10 reads it, half a tick into it); 0.125 s is 7.5, which rounds to 8
  // (tick 12 reads it, half a tick before its start).
  p.commands("0.1:key_down:KeyD;0.1083:key_down:KeyE;0.125:key_down:KeyF");
  engine.advance(12);
  out.seen = seen.map((e) => ({ ...e, at: Math.round(e.at * 1e4) / 1e4 }));
  out.hash = host === "publish" ? "" : engine.hash();
} else if (which === "latency") {
  const Heard = defineResource("CrHeard", { latency: f64(-1) });
  const Listen = defineSystem("CrListen", { reads: [og.Audio], writes: [Heard] }, (_q, _t, world) => {
    const h = world.resource(Heard);
    if (h.ok) h.value.latency = og.audio.outputLatency(world);
  });
  const engine = start(og.game({ tick: 60, resources: [Heard], systems: [Listen] }));
  engine.advance(1);
  const read = () => {
    const h = engine.world.resource(Heard);
    return h.ok ? h.value.latency : null;
  };
  out.headless = read();
  out.outside = og.audio.outputLatency(engine.world);
  if (host !== "node") {
    // The Worker's side of the web host's audio: an AudioContext that says 12 ms (base 5, output 7) at 48 kHz.
    const { WorkerAudio, AUDIO_PRIME_FRAMES } = await import("../../src/web/audio.ts");
    const dir = host === "publish" ? "web-publish" : "web";
    const glue = await import(join(pkg, "dist", dir, "origin-engine.js"));
    const attached = WorkerAudio.attach(glue as never, engine.id, { type: "audio", sampleRate: 48000, latency: 0.012 });
    if (!attached.ok) throw new Error(attached.reason);
    engine.advance(1);
    out.reported = read();
    out.expected = 0.012 + AUDIO_PRIME_FRAMES / 48000;
    attached.value.latency(0.02);
    engine.advance(1);
    out.moved = read();
    out.expectedMoved = 0.02 + AUDIO_PRIME_FRAMES / 48000;
    // The attach started the decoder Worker, which keeps the process alive
    // until it is terminated.
    attached.value.close();
  }
} else {
  throw new Error(`unknown --case ${which}`);
}
console.log(JSON.stringify(out));
