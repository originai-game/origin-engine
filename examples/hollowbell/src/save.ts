// Progress, kept in og.platform's store (og-platform-sdk §2): the key
// `save` holds the run as a small JSON record (the chimes, glimmers, what
// was taken and beaten, the purchases, the last lantern, the bell-keeper's
// name: the save slot's name), written when a lantern is lit and on Save
// and quit, at the revision last read so a stale write is refused and
// merged; `best` holds the best score. Both are asked for on tick 0 and
// shown when they answer (Continue on the title); a later process with the
// same store (`og run --platform-store`, a playtest's `store: "keep"`)
// continues the run.
//
// FINDINGS.md #1, fixed: the engine now saves the whole world
// (world.snapshot, world.restore; og-game §17), about 25 KiB of text for
// this game. Hollowbell keeps this record by choice: a checkpoint game
// restarts at its lantern anyway, the record is a few hundred bytes the
// title reads for the slot's name, it survives any change to the game's
// types, and a restored world would bring back what Game also keeps for
// the session (the platform's request numbers, the best score, the pause
// menu of Save and quit), which Continue would then have to reset.

import { og } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, HEARTS, Phase, Prefs } from "./components.ts";

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

/** What a save holds. */
export interface SaveRecord {
  v: 1;
  name: string;
  glimmers: number;
  chimeMask: number;
  pickedLo: number;
  pickedHi: number;
  heartMask: number;
  killMask: number;
  bossDown: boolean;
  wick: boolean;
  cloak: boolean;
  checkpoint: number;
  litMask: number;
  checkX: number;
  checkY: number;
  time: number;
  hurts: number;
  kills: number;
  hits: number;
  falls: number;
}

function bits(n: number): number {
  let c = 0;
  for (let v = n >>> 0; v !== 0; v >>>= 1) c += v & 1;
  return c;
}

export function record(g: GameV, p: PrefsV): SaveRecord {
  return {
    v: 1,
    name: p.name,
    glimmers: g.glimmers,
    chimeMask: g.chimeMask,
    pickedLo: g.pickedLo,
    pickedHi: g.pickedHi,
    heartMask: g.heartMask,
    killMask: g.killMask,
    bossDown: g.bossDown,
    wick: g.wick,
    cloak: g.cloak,
    checkpoint: g.checkpoint,
    litMask: g.litMask,
    checkX: Math.round(g.checkX * 100) / 100,
    checkY: Math.round(g.checkY * 100) / 100,
    time: Math.round(g.time * 10) / 10,
    hurts: g.hurts,
    kills: g.kills,
    hits: g.hits,
    falls: g.falls,
  };
}

/** The save Continue would restore, or null. */
export function savedRecord(g: GameV): SaveRecord | null {
  if (!g.hasSave || g.saveJson === "") return null;
  try {
    const r = JSON.parse(g.saveJson) as SaveRecord;
    return r && r.v === 1 ? r : null;
  } catch {
    return null;
  }
}

export function chimesIn(r: SaveRecord): number {
  return bits(r.chimeMask);
}

/** A fresh run's progress (the title's world too). */
export function resetProgress(g: GameV): void {
  g.hearts = HEARTS;
  g.glimmers = 0;
  g.chimes = 0;
  g.chimeMask = 0;
  g.pickedLo = 0;
  g.pickedHi = 0;
  g.heartMask = 0;
  g.killMask = 0;
  g.bossDown = false;
  g.bossAwake = false;
  g.wick = false;
  g.cloak = false;
  g.checkpoint = -1;
  g.litMask = 0;
  g.checkX = g.spawnX;
  g.checkY = g.spawnY;
  g.time = 0;
  g.hurts = 0;
  g.kills = 0;
  g.hits = 0;
  g.swings = 0;
  g.falls = 0;
  g.steps = 0;
  g.jumps = 0;
  g.landings = 0;
  g.invulnerable = 0;
  g.knock = 0;
  g.respawn = 0;
  g.ringing = 0;
  g.talking = false;
  g.shopping = false;
  g.paused = false;
  g.newBest = false;
  g.slowTicks = 0;
  g.facing = 1;
}

/** Put a save's progress into the run: Continue. */
export function apply(g: GameV, p: PrefsV, r: SaveRecord): void {
  resetProgress(g);
  p.name = r.name;
  g.glimmers = r.glimmers;
  g.chimeMask = r.chimeMask;
  g.chimes = bits(r.chimeMask);
  g.pickedLo = r.pickedLo;
  g.pickedHi = r.pickedHi;
  g.heartMask = r.heartMask;
  g.killMask = r.killMask;
  g.bossDown = r.bossDown;
  g.wick = r.wick;
  g.cloak = r.cloak;
  g.checkpoint = r.checkpoint;
  g.litMask = r.litMask;
  g.checkX = r.checkpoint >= 0 ? r.checkX : g.spawnX;
  g.checkY = r.checkpoint >= 0 ? r.checkY : g.spawnY;
  g.time = r.time;
  g.hurts = r.hurts;
  g.kills = r.kills;
  g.hits = r.hits;
  g.falls = r.falls;
}

/** Tick 0: ask for the save, the best and the settings. */
export function askRecords(world: World, g: GameV, p: PrefsV): void {
  const save = og.platform.storeGet(world, "save");
  g.loadRequest = save.ok ? save.value : 0;
  const best = og.platform.storeGet(world, "best");
  g.bestRequest = best.ok ? best.value : 0;
  p.loadRequest = og.platform.loadSettings(world);
}

/** Write the run now, at the revision last read or written. */
export function writeSave(world: World, g: GameV, p: PrefsV): void {
  g.saveNow = false;
  const r = record(g, p);
  const asked = og.platform.storeSet(world, { key: "save", value: r, revision: g.saveRevision });
  if (!asked.ok) return;
  g.saveRequest = asked.value;
  g.saveJson = JSON.stringify(r);
  g.hasSave = true;
}

export function writeBest(world: World, g: GameV): void {
  const asked = og.platform.storeSet(world, { key: "best", value: g.best });
  if (asked.ok) g.bestSaveRequest = asked.value;
}

type Answer = { key?: string; value?: unknown; revision?: number } | null;

/** The store's answers, applied when they land. Returns the settings when they answer. */
export function followRecords(world: World, g: GameV, p: PrefsV): Record<string, unknown> | null {
  if (g.saveNow && g.phase === Phase.Play) writeSave(world, g, p);
  if (g.loadRequest !== 0) {
    const r = og.platform.result(world, g.loadRequest);
    if (r !== null) {
      g.loadRequest = 0;
      const data = (r.ok ? r.data : null) as Answer;
      if (data && data.value && typeof data.value === "object" && (data.value as SaveRecord).v === 1) {
        g.saveJson = JSON.stringify(data.value);
        g.hasSave = true;
        g.saveRevision = data.revision ?? 0;
      }
    }
  }
  if (g.bestRequest !== 0) {
    const r = og.platform.result(world, g.bestRequest);
    if (r !== null) {
      g.bestRequest = 0;
      const data = (r.ok ? r.data : null) as Answer;
      if (data && typeof data.value === "number") g.best = Math.max(g.best, data.value);
    }
  }
  if (g.saveRequest !== 0) {
    const r = og.platform.result(world, g.saveRequest);
    if (r !== null) {
      g.saveRequest = 0;
      if (r.ok) {
        g.saves += 1;
        g.saveRevision = ((r.data as Answer)?.revision ?? g.saveRevision) as number;
      } else if (r.error?.code === "conflict") {
        // Another process moved the save on: take its revision and write ours over it (a run only gains).
        g.conflicts += 1;
        g.saveRevision = r.error.revision;
        if (g.phase === Phase.Play) g.saveNow = true;
      }
    }
  }
  if (g.bestSaveRequest !== 0 && og.platform.result(world, g.bestSaveRequest) !== null) g.bestSaveRequest = 0;
  if (p.loadRequest !== 0) {
    const r = og.platform.result(world, p.loadRequest);
    if (r !== null) {
      p.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as Answer)?.value ?? null;
      if (s && typeof s === "object") return s as Record<string, unknown>;
    }
  }
  return null;
}
