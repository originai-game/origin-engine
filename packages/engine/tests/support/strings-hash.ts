// A str field through TypeScript on the host named by --host (node: the napi
// addon), one process per run: a component and a resource with str fields,
// written with world.insert/world.set and a resource setter, read back with
// world.get, through a system's Uint8Array view and the probe's get:, then
// --ticks fixed steps. Prints JSON { host, hash, ... }.
//
//   bun tests/support/strings-hash.ts --host node [--ticks 30]

import { og, defineComponent, defineResource, defineSystem, str, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const host = opt("--host", "node");
const ticks = Number(opt("--ticks", "30"));
if (host !== "node") throw new Error("strings-hash runs on the native host (--host node)");
(await import("../../src/node/index.ts")).loadNativeHost();

export const Badge = defineComponent("StrBadge", { name: str(16), rank: u32(1) });
export const Profile = defineResource("StrProfile", { name: str(48, "Mei"), renames: u32() });

const out: Record<string, unknown> = { host };

/** On tick 5, the profile takes a new name; every tick, the badges' bytes are read through the view. */
const Rename = defineSystem("StrRename", { reads: [Badge], writes: [Profile] }, (q, time, world) => {
  const p = world.resource(Profile);
  if (!p.ok) return;
  if (time.tick === 5) {
    p.value.name = "小灯 Lantern";
    p.value.renames += 1;
  }
  const names: string[] = [];
  for (const a of q.archetypes()) {
    const bytes = a.column(Badge).name;
    out.viewLength = bytes.length;
    out.viewCount = a.count;
    for (let i = 0; i < a.count; i++) {
      const lane = bytes.subarray(i * 16, i * 16 + 16);
      const end = lane.indexOf(0);
      names.push(new TextDecoder().decode(lane.subarray(0, end < 0 ? 16 : end)));
    }
  }
  out.viewNames = names;
});

function start(world: World): void {
  const ids: Entity[] = [];
  for (const name of ["Ada", "小灯", ""]) {
    const e = world.spawn({ name: `badge-${ids.length}` });
    if (!e.ok) throw new Error(e.reason);
    const inserted = world.insert(e.value, Badge, { name, rank: ids.length + 1 });
    if (!inserted.ok) throw new Error(inserted.reason);
    ids.push(e.value);
  }
  const first = ids[0]!;
  out.got = ids.map((e) => {
    const v = world.get(e, Badge);
    return v.ok ? v.value : v.reason;
  });
  // Over the capacity: an error value, nothing truncated, nothing written.
  const long = world.set(first, Badge, { name: "Bartholomew Bold!", rank: 9 });
  out.long = long.ok ? "accepted" : long.reason;
  const after = world.get(first, Badge);
  out.afterLong = after.ok ? after.value : after.reason;
  // Exactly full: 15 bytes of three-byte characters and one more byte.
  const full = world.set(first, Badge, { name: "小灯笼小灯!" });
  out.full = full.ok ? "ok" : full.reason;
  const p = world.resource(Profile);
  if (!p.ok) throw new Error(p.reason);
  out.profileDefault = p.value.name;
  try {
    p.value.name = "x".repeat(49);
    out.resourceLong = "accepted";
  } catch (e) {
    out.resourceLong = (e as Error).message;
  }
  out.profileAfterLong = p.value.name;
}

const game = og.game({ tick: 60, systems: [Rename], resources: [Profile], start }) as unknown as GameToken;
const created = createEngine(game);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const advanced = engine.advance(ticks);
if (!advanced.ok) throw new Error(advanced.reason);
const world = engine.world;
const p = world.resource(Profile);
out.profile = p.ok ? { name: p.value.name, renames: p.value.renames } : p.reason;
// Snapshot and restore: the text is part of the world's bytes.
const hashBefore = engine.hash();
const snap = engine.probe.snapshot() as { id?: string } | string;
const sid = typeof snap === "string" ? snap : String(snap.id ?? "");
const first = world.find("badge-0")!;
world.set(first, Badge, { name: "Zed" });
out.hashChanged = engine.hash() !== hashBefore;
out.restored = engine.probe.restore(sid);
out.hashRestored = engine.hash() === hashBefore;
const back = world.get(first, Badge);
out.afterRestore = back.ok ? back.value : back.reason;
// The probe's command channel: get: reads the text, set: writes it and refuses one over the capacity.
out.probeCommands = engine.probe.commands("0:get:badge-0.StrBadge.name:badge.json;0:set:StrProfile.name=Ada Lovelace;0:set:badge-1.StrBadge.name=Bartholomew Bold!");
engine.advance(1);
out.log = engine.takeLog().filter((l) => l.warning).map((l) => l.text);
out.outputs = engine.takeOutputs().filter((o) => o.kind === "json").map((o) => JSON.parse(o.data) as unknown);
const p2 = world.resource(Profile);
out.probeProfile = p2.ok ? p2.value.name : p2.reason;
out.tick = advanced.value;
out.hash = engine.hash();
console.log(JSON.stringify(out));
