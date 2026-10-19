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
      if (!world.emit(og.KeyUp, { key: "KeyA" }).ok) s.value.refused += 1;
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
} else {
  throw new Error(`unknown --case ${which}`);
}
console.log(JSON.stringify(out));
