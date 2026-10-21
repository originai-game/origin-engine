// What the platform keeps (og.platform), every call a request answered at a
// later tick, its number kept in Game or Prefs:
//
// - The save slot: resting at a waystone writes the whole world
//   (world.snapshot(), og-game §17) in parts (`slot.0`, `slot.1`: it is
//   over a key's 64 KiB), then `slot` with what the title shows of it (the
//   place, the hour) and the parts' count. Continue reads the parts and
//   world.restore()s their text. The world
//   is saved with Game.fromSave set, so the tick after a restore knows it
//   happened and puts back what is the session's, not the save's (the
//   settings and the progress are read again; request numbers cleared).
// - The progress, key `progress`: the chapters reached and their stars, the
//   tutorial seen, the fastest ascent; written at the revision read, a
//   conflict read again and merged (the better of each).
// - The settings (loadSettings/saveSettings).
// - The records: a won ascent's time on the `ascent` board (lower is
//   better), the top and the courier's neighbours read for the Records
//   screen.

import { og } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Phase, Prefs } from "./components.ts";
import { WAYSTONES } from "./layout.ts";
import { flush } from "./state.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

/** The leaderboard the ascents go on: seconds, lower is better. */
export const BOARD = "ascent";

interface Progress {
  reached?: number;
  stars?: number;
  tutorial?: boolean;
  best?: number;
  bestScore?: number;
}

interface Slot {
  place?: number;
  hour?: number;
  parts?: number;
}

function asked(r: { ok: true; value: number } | { ok: false; reason: string }): number {
  return r.ok ? r.value : 0;
}

/** Tick 0: the slot (for the title's Continue), the progress and the settings. */
export function readAll(world: World, g: GameV, p: PrefsV): void {
  g.slotRequest = asked(og.platform.storeGet(world, "slot"));
  g.progressRequest = asked(og.platform.storeGet(world, "progress"));
  p.loadRequest = og.platform.loadSettings(world);
}

/** The most text one store key takes here: the store refuses a value over 64 KiB. */
const PART = 60000;

/**
 * Rests at waystone `i`: the whole world saved in the slot, with the place
 * and the hour. This world snapshots to about 78 KB of text, over the
 * store's 64 KiB a key (og-game §17 puts it in one key; FINDINGS #13), so
 * the text goes in parts (`slot.0`, `slot.1`, ...) and `slot` holds what
 * the title shows and how many parts there are, written last.
 */
export function saveSlot(world: World, g: GameV, i: number): void {
  g.saves += 1;
  g.savePlace = i;
  g.saveClock = g.hour;
  // Marked so the restored world knows it was restored (see the top of this file): what this
  // system changed is written first, then the mark on the resource itself, around the snapshot.
  flush(g);
  const live = must(world.resource(Game));
  live.fromSave = true;
  const snap = world.snapshot();
  live.fromSave = false;
  if (!snap.ok) {
    og.ui.toast(world, snap.reason, { tone: "danger" });
    return;
  }
  const text = snap.value;
  const parts = Math.ceil(text.length / PART);
  g.saveBytes = text.length;
  for (let k = 0; k < parts; k++) {
    const r = og.platform.storeSet(world, { key: `slot.${k}`, value: text.slice(k * PART, (k + 1) * PART) });
    if (!r.ok) {
      og.ui.toast(world, r.reason, { tone: "danger" });
      return;
    }
  }
  const value: Slot = { place: i, hour: Math.round(g.hour * 100) / 100, parts };
  const r = og.platform.storeSet(world, { key: "slot", value });
  if (!r.ok) {
    og.ui.toast(world, r.reason, { tone: "danger" });
    return;
  }
  g.saveRequest = r.value;
  g.hasSave = true;
  g.saveParts = parts;
}

/** Continue: the slot's parts read; the world is restored when every one has landed. */
export function continueRun(world: World, g: GameV): void {
  const n = Math.min(4, g.saveParts);
  g.partRequest0 = n > 0 ? asked(og.platform.storeGet(world, "slot.0")) : 0;
  g.partRequest1 = n > 1 ? asked(og.platform.storeGet(world, "slot.1")) : 0;
  g.partRequest2 = n > 2 ? asked(og.platform.storeGet(world, "slot.2")) : 0;
  g.partRequest3 = n > 3 ? asked(og.platform.storeGet(world, "slot.3")) : 0;
  g.restoring = n > 0;
}

/** The progress as the store should hold it. */
function progressValue(g: GameV): Progress {
  return { reached: g.chaptersReached, stars: g.chapterStars, tutorial: g.tutorialDone, best: g.best, bestScore: g.bestScore };
}

/** Writes the progress at the revision last read (a conflict is read again and merged). */
export function writeProgress(world: World, g: GameV): void {
  g.progressWrite = asked(og.platform.storeSet(world, { key: "progress", value: progressValue(g), revision: g.progressRevision }));
}

/** Merges a stored progress into the run's: the better of each. */
function merge(g: GameV, v: Progress): void {
  g.chaptersReached |= v.reached ?? 0;
  let stars = 0;
  for (let c = 0; c < 4; c++) stars |= Math.max((g.chapterStars >> (c * 2)) & 3, ((v.stars ?? 0) >> (c * 2)) & 3) << (c * 2);
  g.chapterStars = stars;
  g.tutorialDone ||= v.tutorial === true;
  if (typeof v.best === "number" && v.best > 0 && (g.best === 0 || v.best < g.best)) g.best = v.best;
  if (typeof v.bestScore === "number" && v.bestScore > g.bestScore) g.bestScore = v.bestScore;
}

/** A won ascent on the board. */
export function submitAscent(world: World, g: GameV): void {
  g.submitRequest = asked(og.platform.leaderboardSubmit(world, { board: BOARD, score: Math.round(g.time * 10) / 10, order: "asc" }));
}

/** The Records screen's rows for its scope: the fastest ten, or the ten round the courier. */
export function readRecords(world: World, g: GameV): void {
  g.recordsState = 1;
  g.recordsRequest = g.recordsScope === 0 ? og.platform.leaderboardTop(world, { board: BOARD, count: 10 }) : og.platform.leaderboardAround(world, { board: BOARD, count: 10 });
}

/** The answers that landed this tick; what Continue's restore needs back is in `afterRestore`. */
export function answers(world: World, g: GameV, p: PrefsV, applyPrefs: (s: Record<string, unknown>) => void): void {
  const take = (n: number) => (n === 0 ? null : og.platform.result(world, n));
  const slot = take(g.slotRequest);
  if (slot) {
    g.slotRequest = 0;
    const v = (slot.ok ? (slot.data as { value?: Slot }).value : null) ?? null;
    if (v && typeof v.parts === "number" && v.parts > 0) {
      g.hasSave = true;
      g.saveParts = v.parts;
      g.savePlace = Math.max(0, Math.min(WAYSTONES.length - 1, v.place ?? 0));
      g.saveClock = v.hour ?? 0;
    }
  }
  const prog = take(g.progressRequest);
  if (prog) {
    g.progressRequest = 0;
    if (prog.ok) {
      const d = prog.data as { value?: Progress | null; revision?: number };
      g.progressRevision = d.revision ?? 0;
      if (d.value) merge(g, d.value);
    }
  }
  const wrote = take(g.progressWrite);
  if (wrote) {
    g.progressWrite = 0;
    if (wrote.ok) g.progressRevision = (wrote.data as { revision: number }).revision;
    else if (wrote.error?.code === "conflict") {
      // Another device moved it: read it again (merged on arrival) and write the merge.
      g.progressRevision = wrote.error.revision;
      g.conflicts += 1;
      g.progressRequest = asked(og.platform.storeGet(world, "progress"));
      g.progressAgain = true;
    }
  }
  if (g.progressAgain && g.progressRequest === 0) {
    g.progressAgain = false;
    writeProgress(world, g);
  }
  const saved = take(g.saveRequest);
  if (saved) g.saveRequest = 0;
  if (g.restoring) {
    const reqs = [g.partRequest0, g.partRequest1, g.partRequest2, g.partRequest3].filter((n) => n !== 0);
    const got = reqs.map((n) => og.platform.result(world, n));
    if (got.every((r) => r !== null)) {
      g.restoring = false;
      g.partRequest0 = g.partRequest1 = g.partRequest2 = g.partRequest3 = 0;
      const text = got.map((r) => (r!.ok && typeof (r!.data as { value?: unknown }).value === "string" ? ((r!.data as { value: string }).value) : "")).join("");
      const r = world.restore(text);
      if (!r.ok) og.ui.toast(world, r.reason, { tone: "danger" });
    }
  }
  const settings = take(p.loadRequest);
  if (settings) {
    p.loadRequest = 0;
    const s = ((settings.ok ? settings.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
    if (s && typeof s === "object") applyPrefs(s);
  }
  const sub = take(g.submitRequest);
  if (sub) {
    g.submitRequest = 0;
    if (sub.ok) {
      g.submitted += 1;
      const rank = (sub.data as { rank?: number | null }).rank;
      g.rank = typeof rank === "number" ? rank : -1;
    }
  }
  const rows = take(g.recordsRequest);
  if (rows) {
    g.recordsRequest = 0;
    if (!rows.ok) {
      g.recordsState = 3;
      g.records = "";
    } else {
      const entries = (rows.data as { entries?: { rank: number; name?: string; score: number; me?: boolean }[] }).entries ?? [];
      g.records = JSON.stringify(entries.slice(0, 10).map((e) => [e.rank, (e.name ?? "").slice(0, 24), e.score, e.me ? 1 : 0]));
      g.recordsState = 2;
    }
  }
}

/**
 * The tick after a restore (Game.fromSave came back set): what is the
 * session's, not the save's, is put back. The settings and the progress are
 * read again, no request of the saved moment is waited on, and the courier
 * is welcomed back. Answers whether it ran.
 */
export function afterRestore(world: World, g: GameV, p: PrefsV): boolean {
  if (!g.fromSave) return false;
  g.fromSave = false;
  g.loads += 1;
  g.saveRequest = 0;
  g.restoring = false;
  g.partRequest0 = g.partRequest1 = g.partRequest2 = g.partRequest3 = 0;
  g.submitRequest = 0;
  g.recordsRequest = 0;
  g.progressWrite = 0;
  g.progressAgain = false;
  g.progressRequest = asked(og.platform.storeGet(world, "progress"));
  g.slotRequest = asked(og.platform.storeGet(world, "slot"));
  p.loadRequest = og.platform.loadSettings(world);
  g.paused = false;
  g.settingsOpen = false;
  g.bagOpen = false;
  g.phase = Phase.Play;
  return true;
}
