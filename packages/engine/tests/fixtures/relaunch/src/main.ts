// A game that knows how long it was closed: `save` writes the host's time
// (og.platform.now) to the store; a launch reads the save, and the answer's
// time less the save's is the time away. tests/core.test.ts runs its
// playtests with `og test`: across two processes with a kept store and the
// second's clock two hours on, and without a kept store.
import { og, defineResource, defineSystem, f64, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const Away = defineResource("Away", {
  request: u32(),
  loaded: u32(),
  found: u32(),
  savedAt: f64(),
  answeredAt: f64(),
  away: f64(0, { unit: "s" }),
  saves: u32(),
});

const Launch = defineSystem("Launch", { writes: [Away] }, (_q, time, world) => {
  const a = world.resource(Away);
  if (!a.ok) return;
  if (time.tick === 0) {
    const asked = og.platform.storeGet(world, "save");
    a.value.request = asked.ok ? asked.value : 0;
    return;
  }
  if (a.value.request === 0 || a.value.loaded) return;
  const r = og.platform.result(world, a.value.request);
  if (r === null) return;
  a.value.loaded = 1;
  const saved = (r.ok ? (r.data as { value?: { at?: number } | null }).value : null) ?? null;
  if (saved && typeof saved.at === "number") {
    a.value.found = 1;
    a.value.savedAt = saved.at;
    a.value.answeredAt = r.time;
    a.value.away = (r.time - saved.at) / 1000;
  }
});

function save(world: World): void {
  const a = world.resource(Away);
  if (!a.ok) throw new Error(a.reason);
  const w = og.platform.storeSet(world, { key: "save", value: { at: og.platform.now(world) } });
  if (!w.ok) throw new Error(w.reason);
  a.value.saves += 1;
}

export default og.game({ tick: 60, seed: 1, resources: [Away], systems: [Launch], commands: { save } });
