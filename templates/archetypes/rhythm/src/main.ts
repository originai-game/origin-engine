// Three-Lane Beat, the rhythm archetype (2D profile): notes fall down three
// lanes to the hit line in time with "Neon Pulse" (120 BPM, 24 bars); hit
// each as it crosses the line (A S D, the arrows, a finger on the lane, the
// pad). Every hit is judged by how far it was from the note's exact time
// (Perfect, Great, Good), every note let past is a Miss; hits build the
// combo and its multiplier and fill the groove, misses drain it, and an
// empty groove ends the song early. A calibration measures how late this
// device's sound and picture reach the player.
//
// The song is its stems, started on bar lines through og.Music (src/
// systems.ts's Band); the chart is a scene (assets/charts/neon-pulse.ogt)
// of a Song row and a Note row per note. `start` loads them, the style, the
// strings, the sounds and the screen documents (assets/ui/), and spawns
// the clock, the stems, the receptors and the voices. The systems are in
// src/systems.ts, the screens in src/ui.ts and the state in
// src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Game, Glow, LANES, Note, Prefs, Receptor, Rival, Sfx, Song, Sound, Stem, Tuning } from "./components.ts";
import { device, useDevice } from "./device.ts";
import { ONLINE, press, SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { Band, Clock, Controls, Glows, highwayX, LANE_COLOURS, Lanes, PARTS, Play, Voices, worldTime } from "./systems.ts";
import { accuracy, begin, calibrate, LANGUAGES, pause, resume, Ui, winner } from "./ui.ts";

/** The night, linear. */
const NIGHT = { r: 0.002, g: 0.006, b: 0.014 };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
function must(r: { ok: true } | { ok: false; reason: string }): void;
function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function start(world: World): void {
  setupSeats(world);
  useDevice(world);
  // A player per seat, carrying its og.PlayerInput (Seats gives it; Controls reads it).
  for (let seat = 0; seat < SEATS; seat++) {
    const e = must(world.spawn({ name: `seat-${seat}` }));
    must(world.insert(e, og.PlayerInput, { player: seat }));
  }
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/lanebeat/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));

  const settings = must(world.resource(og.RenderSettings));
  settings.environmentIntensity = 0;
  settings.toneMapped = false;
  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = 10;
  camera.position = { x: 0, y: 0, z: 50 };
  camera.background = NIGHT;

  // The chart: a Song row and a Note row per note.
  const chart = must(world.instantiate(must(og.load.scene(world, "charts/neon-pulse.ogt")), { name: "chart" }));
  const songRow = world.find("song");
  if (songRow === null) throw new Error("charts/neon-pulse.ogt has no entity named song");
  const song = must(world.get(songRow, Song));
  const game = must(world.resource(Game));
  game.bpm = song.bpm;
  game.bars = song.bars;
  game.notes = song.notes;
  // Versus: seat 1 plays the same chart on its own highway, its own copy of every note.
  if (SEATS > 1) {
    must(world.resource(Rival)).notes = song.notes;
    for (const e of world.children(chart)) {
      const n = world.get(e, Note);
      if (!n.ok) continue;
      must(world.insert(must(world.spawn({ name: `note-${n.value.index}@1` })), Note, { index: n.value.index, lane: n.value.lane, beat: n.value.beat, seat: 1 }));
    }
  }
  const texture = (path: string) => must(og.load.texture(world, path));
  game.noteTexture = texture("sprites/note.png") as unknown as number;

  // The clock the stems start on (og.Music; the Clock system keeps it on Game.songStart).
  const clock = must(world.spawn({ name: "clock" }));
  must(world.insert(clock, og.Music, { bpm: song.bpm, beatsPerBar: song.beatsPerBar, start: 1e6 }));
  PARTS.forEach((part, i) => {
    const e = must(world.spawn({ name: `stem-${part}` }));
    const click = part.startsWith("click");
    must(world.insert(e, og.AudioSource, { sound: sound(`audio/${click ? "click" : part}.wav`), playing: false, bus: click ? "ui" : "music", volume: click ? 0.6 : part === "drums" ? 0.8 : 0.6, priority: 240, fade: 0.004 }));
    must(world.insert(e, Stem, { part: i }));
  });

  // The highway: three lanes' glow, the hit line's receptors (with their bursts).
  const lane = texture("sprites/lane.png");
  const ring = texture("sprites/ring.png");
  const hit = must(og.load.emitter(world, "fx/hit.ogt"));
  const game0 = must(world.resource(Game));
  for (let seat = 0; seat < SEATS; seat++) {
    const x0 = highwayX(game0, seat);
    const tag = seat === 0 ? "" : `@${seat}`;
    for (let i = 0; i < LANES; i++) {
      const c = LANE_COLOURS[i]!;
      const glow = must(world.spawn({ name: `lane-${i}${tag}`, position: { x: x0 + (i - 1) * 1.5, y: -0.2, z: 0 } }));
      must(world.insert(glow, og.Sprite, { texture: lane, width: 1.4, height: 9.6, color: c, opacity: 0.22, blend: "additive", layer: 1 }));
      must(world.insert(glow, Glow, { lane: i, seat }));
      const r = must(world.spawn({ name: `receptor-${i}${tag}`, position: { x: x0 + (i - 1) * 1.5, y: -3.2, z: 0 } }));
      must(world.insert(r, og.Sprite, { texture: ring, width: 1.29, height: 0.54, color: c, opacity: 0.6, layer: 5, visible: false }));
      must(world.insert(r, og.ParticleSystem, { emitter: hit, seed: seat * LANES + i + 1, visible: false }));
      must(world.insert(r, Receptor, { lane: i, seat }));
    }
  }

  const voices: [string, number, "sfx" | "ui", number][] = [
    ["audio/tap0.wav", Sound.Tap0, "ui", 0.45],
    ["audio/tap1.wav", Sound.Tap1, "ui", 0.45],
    ["audio/tap2.wav", Sound.Tap2, "ui", 0.45],
    ["audio/miss.wav", Sound.Miss, "sfx", 0.5],
    ["audio/miss.wav", Sound.Break, "sfx", 0.8],
    ["audio/clear.wav", Sound.Clear, "sfx", 0.7],
    ["audio/fail.wav", Sound.Fail, "sfx", 0.7],
    ["audio/ui-press.ogt", Sound.Move, "ui", 0.3],
  ];
  for (const [path, bit, bus, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus, volume, priority: 200 }));
    must(world.insert(v, Sfx, { bit }));
  }
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `play[:<bar>]`: start the song now, as Play does; from a later bar, the notes before it skipped. */
function play(world: World, args: readonly string[]): void {
  begin(world, gameOf(world), args[0] === undefined ? 1 : num(args[0], "bar"));
}

/** `hit:<lane>[:<seat>]`: press a lane now, as a key or a finger does (seat 0, or a local seat). */
function hit(_world: World, args: readonly string[]): void {
  const lane = num(args[0], "lane");
  if (lane < 0 || lane >= LANES) throw new Error("the lane is 0, 1 or 2");
  const seat = args[1] === undefined ? 0 : num(args[1], "seat");
  if (seat !== 0 && seat !== 1) throw new Error("the seat is 0 or 1");
  press(`lane${lane}`, seat);
}

/** `groove:<0-1>`: set the groove. */
function groove(world: World, args: readonly string[]): void {
  gameOf(world).groove = num(args[0], "groove");
}

/** `offset:<ms>`: set this device's calibrated offset. */
function offset(world: World, args: readonly string[]): void {
  device.offsetMs = num(args[0], "ms");
  if (!ONLINE) must(world.resource(Prefs)).offsetMs = device.offsetMs;
}

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`. */
function language(world: World, args: readonly string[]): void {
  const p = must(world.resource(Prefs));
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  p.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

/** Versus facts: seat 1's score, combo, groove and judgements, and who leads. */
function versus(world: World) {
  const r = world.resource(Rival);
  const g = world.resource(Game);
  if (!r.ok || !g.ok) return {};
  const v = r.value;
  return { rival: { score: v.score, combo: v.combo, maxCombo: v.maxCombo, perfect: v.perfect, great: v.great, good: v.good, miss: v.miss, judged: v.judged, groove: v.groove, failed: v.failed, accuracy: accuracy(v) }, leader: winner([g.value, v]) };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Rival, Tuning, Prefs],
  systems: [Seats, Controls, Ui, Clock, Play, Band, Lanes, Glows, Voices],
  render: { antialias: "msaa4" },
  start,
  commands: {
    play,
    hit,
    groove,
    offset,
    calibrate: (world: World) => {
      if (ONLINE) throw new Error("calibration is off in an online game (the offset is each device's own)");
      calibrate(world, gameOf(world));
    },
    hold: (world: World) => pause(gameOf(world), worldTime(world)),
    unhold: (world: World) => resume(world, gameOf(world)),
    viewport,
    language,
    bot: (_world: World, args: readonly string[]) => setBot(num(args[0], "bot")),
  },
  commandHelp: {
    play: { usage: "[<bar>]", doc: "Start the song now, as Play does (the count-in, then bar 1); from a later bar, the earlier notes skipped." },
    hit: { usage: "<lane 0-2>[:<seat>]", doc: "Press a lane now, as a key or a finger does (seat 0 unless named); its seat's input carries it the next tick." },
    groove: { usage: "<0-1>", doc: "Set the groove (the life gauge)." },
    offset: { usage: "<ms>", doc: "Set the calibrated offset." },
    calibrate: { usage: "", doc: "Open the calibration, as Calibrate does." },
    hold: { usage: "", doc: "Pause the play, as Escape does." },
    unhold: { usage: "", doc: "Resume the play, as Resume does (on its phrase's bar line, after a count-in)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    bot: { usage: "<n>", doc: "A scripted player for this device's seats (0 hands the input back): src/seats.ts's bot." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "results", "calibrate"][g.phase],
      time: worldTime(world) - g.songStart,
      score: g.score,
      combo: g.combo,
      maxCombo: g.maxCombo,
      perfect: g.perfect,
      great: g.great,
      good: g.good,
      miss: g.miss,
      judged: g.judged,
      notes: g.notes,
      groove: g.groove,
      accuracy: accuracy(g),
      cleared: g.cleared,
      failed: g.failed,
      best: g.best,
      language: LANGUAGES[g.language]!.code,
      ...(SEATS > 1 ? versus(world) : {}),
    };
  },
});

