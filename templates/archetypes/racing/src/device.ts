// This device's own, never in the world: the player's settings (the
// volumes, the language, reduced motion) and the record of races run here
// ({races, wins, best lap}). Settings are loaded from og.platform's
// settings on the first ticks and saved by the settings' Done; the record
// is read from the store on the first tick and written at the revision
// read. They change presentation alone (the buses, the locale, the
// camera's shake, the screens' dissolve) and what this device keeps, so in
// a room each player keeps their own and nothing of theirs enters the
// world or its hash. src/ui.ts loads, applies and saves them.

/** The player's settings on this device, its settings request, and how often Done saved them. */
export const prefs = { master: 1, music: 0.6, effects: 1, language: 0, reducedMotion: false, loadRequest: 0, saves: 0 };

/**
 * This device's record and its store bookkeeping: what it holds, the
 * request in flight, the revision last read or written, and what the store
 * held then (so a merge adds only what this device added since).
 */
export const record = {
  races: 0,
  wins: 0,
  /** The best lap run on this device, in ticks (0 none). */
  bestLap: 0,
  /** This race set it. */
  newBest: false,
  request: 0,
  write: 0,
  revision: 0,
  merge: false,
  dirty: false,
  conflicts: 0,
  storedRaces: 0,
  storedWins: 0,
  sentRaces: 0,
  sentWins: 0,
};

/** Whether the settings dialog is open on this device. */
export const screens = { settingsOpen: false };
