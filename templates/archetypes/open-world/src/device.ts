// This device's own, never in the world: the player's settings (the
// volumes, the language, reduced motion) and the record (runs, wins, the
// best time), kept in og.platform. They change presentation alone (the
// buses in og.Audio, each peer's own; the strings' locale; the camera's
// shake and the screens' dissolve) or count this device's runs, so in a
// room each player keeps their own and nothing here enters the world or
// its hash. src/ui.ts loads, applies and saves them.

/** The player's settings on this device, its settings request, how often Done saved them, and whether an answer brought new ones (the buses take them). */
export const prefs = { master: 1, music: 0.7, effects: 1, language: 0, reducedMotion: false, loadRequest: 0, saves: 0, fresh: false };

/**
 * The record on this device: what it holds, and the bookkeeping of its
 * store calls (one at a time; a write refused as stale is read again and
 * the merge written at the new revision).
 */
export const record = {
  runs: 0,
  wins: 0,
  /** The fastest win in seconds of play (0 none). */
  best: 0,
  newBest: false,
  request: 0,
  revision: 0,
  write: 0,
  merge: false,
  dirty: false,
  /** The counts as last read or written: runs less this is what this device added. */
  storedRuns: 0,
  storedWins: 0,
  sentRuns: 0,
  sentWins: 0,
  conflicts: 0,
  /** The run counted here last (a run is counted once, on the tick it starts). */
  counted: -1,
};
