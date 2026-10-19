// Every piece of Three-Lane Beat's state lives in these reflected types:
// dumps, the probe, Tune and the command channel's `get` and `set` read and
// write them by name.
//
// The play (Game), the knobs (Tuning) and the player's settings with the
// calibrated offset (Prefs) are resources. The chart is a scene
// (assets/charts/*.ogt): its `Song` row and one `Note` row per note. The
// stems, the lanes' receptors and the voices are rows spawned in `start`.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Results: 2, Calibrate: 3 } as const;

/** A note's judgement (Note.judged - 1, Game.lastJudgement). */
export const Judgement = { Perfect: 0, Great: 1, Good: 2, Miss: 3 } as const;
export const JUDGEMENTS = ["perfect", "great", "good", "miss"] as const;

/** The lanes. */
export const LANES = 3;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Tap0: 1, Tap1: 2, Tap2: 4, Miss: 8, Clear: 16, Fail: 32, Move: 64, Break: 128 } as const;

/** The play: the song's clock, the judgements, the score, the groove, the calibration, the best. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  songStart: f64(0, { unit: "s", doc: "The world time of bar 1's first beat (og.Music.start): every note's time is songStart + beat × 60 / bpm." }),
  pausedAt: f64(-1, { unit: "s", doc: "The world time the play was paused at (-1 not paused)." }),
  countTo: f64(0, { unit: "s", doc: "The count-in ends at this world time (after a start or a resume)." }),
  score: u32(0),
  combo: u32(0),
  maxCombo: u32(0),
  perfect: u32(0),
  great: u32(0),
  good: u32(0),
  miss: u32(0),
  hits: u32(0, { doc: "Notes hit (perfect, great or good)." }),
  judged: u32(0, { doc: "Notes judged (hit or missed)." }),
  notes: u32(0, { doc: "Notes in the chart." }),
  taps: u32(0, { doc: "Lane presses." }),
  groove: f32(0.5, { doc: "The life gauge, 0 to 1: hits fill it, misses drain it; empty fails the song." }),
  failed: bool(false),
  cleared: bool(false),
  lastJudgement: i32(-1, { doc: "Judgement.* of the last judged note." }),
  lastOffsetMs: f32(0, { unit: "ms", doc: "The last hit's error: negative early, positive late." }),
  lastLane: i32(-1),
  scheduled: u32(0, { doc: "Notes put on the highway so far by og.MusicBeat (the next note to schedule)." }),
  nextBar: u32(0, { doc: "The bar whose stems the Band starts next." }),
  endAt: f64(0, { unit: "s", doc: "The world time the results come up (a beat after the last note, or after the groove ran out)." }),
  calTaps: u32(0, { doc: "Calibration taps so far (of 8)." }),
  calSum: f64(0, { unit: "ms", doc: "The calibration taps' errors, summed." }),
  best: u32(0),
  bestRequest: u32(0),
  bestRevision: u32(0),
  bestWrite: u32(0),
  newBest: bool(false),
  padLane: i32(-1, { doc: "A lane a command pressed this tick (-1 none)." }),
  bpm: f32(120, { unit: "bpm", doc: "The chart's tempo (its Song row)." }),
  bars: u32(0, { doc: "The chart's length in bars." }),
  pressMask: u32(0, { doc: "The lanes pressed this tick (bit per lane)." }),
  hitMask: u32(0, { doc: "The lanes whose press hit a note this tick." }),
  resetNotes: bool(false, { doc: "A new play: the Play system clears every note's judgement this tick." }),
  fromBeat: f32(0, { unit: "beats", doc: "Where this play started (play:<bar> starts later): earlier notes are skipped, not judged." }),
  scheduleUpTo: f32(-1, { unit: "beats", doc: "A start or a resume puts the notes up to this beat on the highway (-1 none); og.MusicBeat does the rest." }),
  countShown: i32(-1, { doc: "The count-in number last shown." }),
  over: bool(false, { doc: "The play is over: the Ui system shows the results." }),
  stopBand: bool(false, { doc: "Stop every stem this tick (a pause, a fail, the title)." }),
  bandFrom: u32(0, { doc: "A start or a resume: the bar the Band starts at Game.countTo, after the count-in's clicks (0 none)." }),
  noteTexture: f64(0, { doc: "The note's texture handle." }),
  sfx: u32(0),
  laneW: f32(1.5, { unit: "m", doc: "A lane's width (src/ui.ts sets the layout for the surface)." }),
  hitY: f32(-3.2, { unit: "m", doc: "The hit line's height." }),
  topY: f32(4.8, { unit: "m", doc: "Where notes appear." }),
  viewW: f32(0, { unit: "cssPx" }),
  viewH: f32(0, { unit: "cssPx" }),
  language: u32(0),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  perfectMs: f32(45, { min: 5, max: 200, step: 1, unit: "ms", tunable: true, label: "Perfect window", doc: "A hit within this of the note's time (either side) is Perfect." }),
  greatMs: f32(90, { min: 5, max: 250, step: 1, unit: "ms", tunable: true, label: "Great window" }),
  goodMs: f32(135, { min: 5, max: 300, step: 1, unit: "ms", tunable: true, label: "Good window", doc: "Past this late, an unhit note is a Miss." }),
  scrollBeats: f32(2, { min: 0.5, max: 8, step: 0.25, unit: "beats", tunable: true, label: "Scroll", doc: "How many beats ahead a note appears at the top." }),
  countInBeats: u32(4, { min: 1, max: 4, tunable: true, label: "Count-in", doc: "Beats of clicks before the song, and before it resumes." }),
  grooveStart: f32(0.5, { min: 0.05, max: 1, step: 0.05, tunable: true, label: "Groove at the start" }),
  grooveGain: f32(0.03, { min: 0, max: 0.5, step: 0.005, tunable: true, label: "Groove per hit" }),
  grooveLoss: f32(0.1, { min: 0, max: 1, step: 0.01, tunable: true, label: "Groove per miss" }),
  perfectPoints: u32(300, { min: 0, max: 10000, tunable: true }),
  greatPoints: u32(200, { min: 0, max: 10000, tunable: true }),
  goodPoints: u32(100, { min: 0, max: 10000, tunable: true }),
  comboStep: u32(10, { min: 1, max: 100, tunable: true, label: "Combo per multiplier", doc: "Every this many in a row raises the multiplier by one, up to four." }),
});

/** The player's settings: kept in og.platform's settings, restored on the first ticks. */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.8, { min: 0, max: 1 }),
  effects: f32(0.8, { min: 0, max: 1 }),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  offsetMs: f32(0, { min: -250, max: 250, unit: "ms", doc: "How late this device's sound and picture reach the player (calibrated): a press is judged this much earlier." }),
  loadRequest: u32(0),
  saves: u32(0),
});

/** The chart's song: tempo, metre and length (one row, from the chart scene). */
export const Song = defineComponent("Song", {
  bpm: f32(120, { unit: "bpm" }),
  beatsPerBar: u32(4),
  bars: u32(0),
  notes: u32(0),
});

/** A note of the chart (a row from the chart scene): its lane and beat, then how it went. */
export const Note = defineComponent("Note", {
  index: u32(0, { doc: "Its place in the chart, in time order." }),
  lane: u32(0, { doc: "0 left, 1 middle, 2 right." }),
  beat: f32(0, { unit: "beats", doc: "From 0 at bar 1's first beat." }),
  live: bool(false, { doc: "On the highway (scheduled by og.MusicBeat, not yet judged)." }),
  judged: u32(0, { doc: "0 not yet, else Judgement.* + 1 (5: skipped, before a play that started later)." }),
  errorMs: f32(0, { unit: "ms", doc: "How early (negative) or late it was hit." }),
});

/** A stem of the song: started on its bar lines by the Band. */
export const Stem = defineComponent("Stem", {
  part: u32(0, { doc: "PARTS index in src/systems.ts." }),
  starts: u32(0, { doc: "Times started." }),
});

/** A lane's receptor at the hit line: it flashes and squashes on a press. */
export const Receptor = defineComponent("Receptor", {
  lane: u32(0),
  presses: u32(0),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
