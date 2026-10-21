// The replay: a finished race driven again, from the engine's determinism.
//
// During the countdown, with the car settled on the grid, the whole world
// is saved (world.snapshot(), in the slot `replay` through
// og.platform.saveSlot); at the finish the run's inputs (two characters a
// tick, src/tape.ts) are stored beside it (`replay.tape`). Watch replay
// (the results) restores that world (world.restore) and the Tapes system
// feeds the taped inputs back to the car tick by tick: the same world and
// the same inputs give the same race, so the replay crosses the line on
// the tick the race did (Race.replayMatched, tests/replay.og.ts).
//
// The snapshot is taken by the first system of a tick (src/ui.ts): taken
// further into a tick, the restored world skips the rest of that tick and
// the replay runs a different race (FINDINGS #12). It is about 400 KB, the
// circuit's triangle-mesh colliders being most of it (FINDINGS #11).
//
// The world saved carries Race.replayMark, set around the snapshot, so the
// tick after the restore knows it was one: it holds the clock (time scale
// 0) while the inputs are read again (request numbers are not carried
// across a restore: the store is asked afresh), then lets the countdown
// run on.

import { og } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Race, RUN_TICKS } from "./components.ts";
import { validTape } from "./tape.ts";
import { readTape, writeTape } from "./tapes.ts";

type RaceV = Value<typeof Race>;

/** The countdown tick the world is saved on: a second and a half before GO, the car long settled. */
export const SNAPSHOT_AT = 90;

const SLOT = "replay";
const TAPE_KEY = "replay.tape";

function asked(r: { ok: true; value: number } | { ok: false; reason: string }): number {
  return r.ok ? r.value : 0;
}

/** In the countdown: the world as it stands, marked as a replay's start, into the slot. */
export function saveStart(world: World, r: RaceV): void {
  r.replayReady = false;
  r.replaySlotOk = false;
  r.replayMark = true;
  const snap = world.snapshot();
  r.replayMark = false;
  if (!snap.ok) return;
  r.snapBytes = snap.value.length;
  r.replaySave = asked(og.platform.saveSlot(world, { slot: SLOT, text: snap.value, meta: { bytes: snap.value.length } }));
}

/** At the finish: the run's inputs and its total, beside the saved start. */
export function saveTape(world: World, r: RaceV): void {
  const inputs = readTape(world, "run", r.runLen * 2);
  r.replayTapeWrite = asked(og.platform.storeSet(world, { key: TAPE_KEY, value: { ticks: r.runLen, total: r.total, inputs } }));
}

/** Watch replay: the saved start is read; the world is restored when it lands. */
export function watch(world: World, r: RaceV): void {
  r.replayLoad = asked(og.platform.loadSlot(world, { slot: SLOT }));
}

/** The replay's answers, each tick (the Ui system). */
export function replayAnswers(world: World, r: RaceV): void {
  const take = (n: number) => (n === 0 ? null : og.platform.result(world, n));
  const saved = take(r.replaySave);
  if (saved) {
    r.replaySave = 0;
    r.replaySlotOk = saved.ok;
  }
  const wrote = take(r.replayTapeWrite);
  if (wrote) {
    r.replayTapeWrite = 0;
    r.replayReady = wrote.ok && r.replaySlotOk;
  }
  const loaded = take(r.replayLoad);
  if (loaded) {
    r.replayLoad = 0;
    const text = loaded.ok ? (loaded.data as { text?: string | null }).text : null;
    const back = typeof text === "string" ? world.restore(text) : { ok: false as const, reason: loaded.error?.message ?? "no replay saved" };
    if (!back.ok) og.ui.toast(world, back.reason, { tone: "danger" });
  }
  // The tick after the restore: the world is the countdown's again. Hold it and read the inputs.
  if (r.replayMark) {
    r.replayMark = false;
    r.replaying = true;
    r.replayWait = true;
    r.replayMatched = 0;
    r.replays += 1;
    r.replayTapeRead = asked(og.platform.storeGet(world, TAPE_KEY));
  }
  const tape = take(r.replayTapeRead);
  if (tape) {
    r.replayTapeRead = 0;
    const v = tape.ok ? ((tape.data as { value?: { ticks?: number; total?: number; inputs?: string } | null }).value ?? null) : null;
    const ticks = v?.ticks ?? 0;
    if (v && typeof v.inputs === "string" && ticks > 0 && ticks <= RUN_TICKS && validTape(v.inputs, ticks)) {
      writeTape(world, "run", v.inputs);
      r.replayLen = ticks;
      r.replayTotal = v.total ?? 0;
      r.replayWait = false;
    } else {
      // Nothing to drive: back to a plain race.
      r.replaying = false;
      r.replayWait = false;
      og.ui.toast(world, og.i18n.t(world, "toast.noReplay"), { tone: "warn" });
    }
  }
}
