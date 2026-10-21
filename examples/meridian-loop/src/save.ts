// What the platform keeps (og.platform), every call a request answered at a
// later tick, its number kept in Race or Prefs (og-platform-sdk §1):
//
// - The record, key `record`: the best lap's time, its three sectors and
//   its poses (the ghost: about 16 KB of text for a 35 s lap), written at
//   the revision read when a lap beats it; a conflict (another device
//   wrote a record meanwhile) is read again and the faster kept.
// - The settings (loadSettings/saveSettings), the language and the ghost
//   switch with them.
// - The `laps` board: the run's best lap in seconds, lower first; the
//   Leaderboard screen reads the top ten or the ten round the player.

import { og } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Prefs, Race, TAPE_TICKS } from "./components.ts";
import { readTape, writeTape } from "./tapes.ts";
import { POSE_BYTES, POSE_EVERY } from "./tape.ts";
import { must } from "./world.ts";

type RaceV = Value<typeof Race>;

/** The board the laps go on: seconds, lower is better. */
export const BOARD = "laps";

interface Record {
  ticks?: number;
  s1?: number;
  s2?: number;
  s3?: number;
  len?: number;
  poses?: string;
}

function asked(r: { ok: true; value: number } | { ok: false; reason: string }): number {
  return r.ok ? r.value : 0;
}

/** Tick 0: the record and the settings. */
export function readAll(world: World, r: RaceV, p: Value<typeof Prefs>): void {
  r.recordRequest = asked(og.platform.storeGet(world, "record"));
  p.loadRequest = og.platform.loadSettings(world);
}

/** A new record: its poses with it, at the revision read. */
export function writeRecord(world: World, r: RaceV, poses: string, len: number): void {
  const value: Record = { ticks: r.record, s1: r.recordS1, s2: r.recordS2, s3: r.recordS3, len, poses };
  r.recordWrite = asked(og.platform.storeSet(world, { key: "record", value, revision: r.recordRevision }));
  r.saves += 1;
}

/** The run's best lap on the board. */
export function submitLap(world: World, r: RaceV, ticks: number): void {
  r.submitRequest = asked(og.platform.leaderboardSubmit(world, { board: BOARD, score: Math.round((ticks * 1000) / 60) / 1000, order: "asc" }));
}

/** The Leaderboard screen's rows for its scope: the fastest ten, or the ten round the player. */
export function readBoard(world: World, r: RaceV): void {
  r.boardState = 1;
  r.boardRequest = r.boardScope === 0 ? og.platform.leaderboardTop(world, { board: BOARD, count: 10 }) : og.platform.leaderboardAround(world, { board: BOARD, count: 10 });
}

/** A stored record into the run, when it is faster than what the run has. */
function takeRecord(world: World, r: RaceV, v: Record): void {
  const ticks = v.ticks ?? 0;
  const len = v.len ?? 0;
  const poses = typeof v.poses === "string" ? v.poses : "";
  if (ticks <= 0 || len <= 0 || len * POSE_EVERY > TAPE_TICKS || poses.length !== len * POSE_BYTES) return;
  if (r.record !== 0 && r.record <= ticks) return;
  r.record = ticks;
  r.recordS1 = v.s1 ?? 0;
  r.recordS2 = v.s2 ?? 0;
  r.recordS3 = v.s3 ?? 0;
  writeTape(world, "best", poses);
  r.bestLen = len;
  r.loaded = true;
}

/** The answers that landed this tick. */
export function answers(world: World, r: RaceV, p: Value<typeof Prefs>, settings: (s: Partial<globalThis.Record<string, unknown>>) => void): void {
  const take = (n: number) => (n === 0 ? null : og.platform.result(world, n));
  const rec = take(r.recordRequest);
  if (rec) {
    r.recordRequest = 0;
    if (rec.ok) {
      const d = rec.data as { value?: Record | null; revision?: number };
      r.recordRevision = d.revision ?? 0;
      if (d.value) takeRecord(world, r, d.value);
      if (r.recordAgain) {
        r.recordAgain = false;
        // Ours is still faster: write it over the one that beat us to the key.
        if (r.record > 0 && r.bestLen > 0 && r.newRecord) writeRecord(world, r, readTape(world, "best", r.bestLen * POSE_BYTES), r.bestLen);
      }
    }
  }
  const wrote = take(r.recordWrite);
  if (wrote) {
    r.recordWrite = 0;
    if (wrote.ok) r.recordRevision = (wrote.data as { revision: number }).revision;
    else if (wrote.error?.code === "conflict") {
      r.conflicts += 1;
      r.recordRevision = wrote.error.revision ?? r.recordRevision;
      r.recordRequest = asked(og.platform.storeGet(world, "record"));
      r.recordAgain = true;
    }
  }
  const s = take(p.loadRequest);
  if (s) {
    p.loadRequest = 0;
    const v = ((s.ok ? s.data : null) as { value?: globalThis.Record<string, unknown> } | null)?.value ?? null;
    if (v && typeof v === "object") settings(v);
  }
  const sub = take(r.submitRequest);
  if (sub) {
    r.submitRequest = 0;
    if (sub.ok) {
      const rank = (sub.data as { rank?: number | null }).rank;
      r.rank = typeof rank === "number" ? rank : -1;
    }
  }
  const rows = take(r.boardRequest);
  if (rows) {
    r.boardRequest = 0;
    if (!rows.ok) {
      r.boardState = 3;
      r.board = "";
    } else {
      const entries = (rows.data as { entries?: { rank: number; name?: string; score: number; me?: boolean }[] }).entries ?? [];
      const packed = entries.slice(0, 10).map((e) => [e.rank, (e.name ?? "").slice(0, 16), e.score, e.me ? 1 : 0]);
      // The rows as text in a str(1024): rows are dropped until it surely fits (three bytes a UTF-16 unit at most).
      let text = JSON.stringify(packed);
      while (text.length * 3 > 1024 && packed.length > 0) {
        packed.pop();
        text = JSON.stringify(packed);
      }
      r.board = text;
      r.boardState = 2;
    }
  }
}
