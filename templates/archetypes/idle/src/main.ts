// Spark Orb, the idle archetype (2D profile): a glowing orb in the night.
// Tap it for sparks; spend them on generators (embers, storms, crystals,
// stars) that make sparks on their own and circle the orb as motes; every
// tenth of a kind doubles that kind. When the run has made enough, the orb
// can collapse into a supernova: sparks and generators go, stardust stays,
// and each stardust makes everything 10 % stronger. The save lives in
// og.platform's store with the host's time it was written at; on a launch
// the time away (the save's answer's time less that) is credited at half
// the rate.
//
// The look (GameDesc.look, a file under assets/looks/) sets the frame's
// treatment and the UI skin before start runs. `start` loads the strings,
// the sounds and the orb's art and spawns the rows: the orb and its glow,
// 48 motes (12 per generator), the burst pools, a voice per sound and the
// music. The systems are in
// src/systems.ts, the formulas and big numbers in src/numbers.ts, the
// screens and the save in src/ui.ts and the state in src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Burst, Game, GENERATORS, Mote, MOTES, Orb, Phase, Prefs, Sfx, Sound, Tuning } from "./components.ts";
import { counts, Bursts, BURST, Economy, lazyTuning, Motes, nova, ORB_RADIUS, OrbTap, TIER_COLOURS, Voices } from "./systems.ts";
import { setOwned } from "./numbers.ts";
import { LANGUAGES, launch, Ui } from "./ui.ts";

/** The night, linear. */
const NIGHT = { r: 0.004, g: 0.002, b: 0.012 };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
function must(r: { ok: true } | { ok: false; reason: string }): void;
function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function start(world: World): void {
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  // GameDesc.look set the frame and loaded the skin before start; the UI
  // keeps that skin and adds the sounds.
  const skin = og.render.look(world)?.skin ?? null;
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { skin, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));

  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = 10;
  camera.position = { x: 0, y: 0, z: 50 };
  camera.background = NIGHT;

  const texture = (path: string) => must(og.load.texture(world, path));
  // The night: a violet haze and fixed stars of the dice (og.Rng's "stars" stream, the same every launch).
  const haze = must(world.spawn({ name: "haze", position: { x: 0, y: 0.35, z: 0 } }));
  must(world.insert(haze, og.Sprite, { texture: texture("sprites/glow.png"), width: 24, height: 18, color: { r: 0.4, g: 0.2, b: 0.85 }, opacity: 0.16, blend: "additive", layer: -11 }));
  const spark = texture("sprites/spark.png");
  const dice = must(world.resource(og.Rng)).fork("stars");
  for (let i = 0; i < 140; i++) {
    const star = must(world.spawn({ name: `star-${i}`, position: { x: dice.range(-15, 15), y: dice.range(-10, 10), z: 0 } }));
    const size = dice.range(0.05, 0.16);
    const warm = dice.next();
    must(world.insert(star, og.Sprite, { texture: spark, width: size, height: size, color: { r: 0.75 + 0.25 * warm, g: 0.82 + 0.1 * warm, b: 1 - 0.25 * warm }, opacity: dice.range(0.35, 0.9), blend: "additive", layer: -10 }));
  }
  const stars = must(world.spawn({ name: "stars", position: { x: 0, y: 0, z: 0 } }));
  must(world.insert(stars, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/stars.ogt")), seed: 3 }));
  const glow = must(world.spawn({ name: "glow", position: { x: 0, y: 0.35, z: 0 } }));
  must(world.insert(glow, og.Sprite, { texture: texture("sprites/glow.png"), width: ORB_RADIUS * 5.5, height: ORB_RADIUS * 5.5, color: { r: 1, g: 0.6, b: 0.2 }, opacity: 0.85, blend: "additive", layer: 1 }));
  const drift = must(world.spawn({ name: "drift", position: { x: 0, y: 0.35, z: 0 } }));
  must(world.insert(drift, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/drift.ogt")), seed: 5 }));
  const orb = must(world.spawn({ name: "orb", position: { x: 0, y: 0.35, z: 0 } }));
  must(world.insert(orb, og.Sprite, { texture: texture("sprites/orb.png"), width: ORB_RADIUS * 2.2, height: ORB_RADIUS * 2.2, layer: 3 }));
  must(world.insert(orb, Orb));

  const moteTextures = GENERATORS.map((name) => texture(`sprites/mote-${name}.png`));
  GENERATORS.forEach((_, tier) => {
    for (let k = 0; k < MOTES; k++) {
      const m = must(world.spawn({ name: `mote-${tier}-${k}`, position: { x: 0, y: -40, z: 0 } }));
      const c = TIER_COLOURS[tier]!;
      must(world.insert(m, og.Sprite, { texture: moteTextures[tier]!, width: 0.42, height: 0.42, color: c, blend: "additive", layer: 4, visible: false }));
      must(world.insert(m, Mote, { tier, index: k }));
    }
  });
  const pools: [number, string, number][] = [[BURST.Tap, "fx/tap.ogt", 6], [BURST.Milestone, "fx/milestone.ogt", 2], [BURST.Nova, "fx/nova.ogt", 1]];
  for (const [kind, path, n] of pools) {
    const emitter = must(og.load.emitter(world, path));
    for (let i = 0; i < n; i++) {
      const b = must(world.spawn({ name: `burst-${kind}-${i}`, position: { x: 0, y: -40, z: 0 } }));
      must(world.insert(b, og.ParticleSystem, { emitter, seed: kind * 10 + i + 1, visible: false }));
      must(world.insert(b, Burst, { kind }));
    }
  }

  const voices: [string, number, "sfx" | "ui", number][] = [
    // The orb is the game's one big button: its chime is on the ui bus, as a press's sound (og-audio §3).
    ["audio/tap.wav", Sound.Tap, "ui", 0.5],
    ["audio/buy.wav", Sound.Buy, "sfx", 0.55],
    ["audio/milestone.wav", Sound.Milestone, "sfx", 0.6],
    ["audio/nova.wav", Sound.Nova, "sfx", 0.75],
    ["audio/ui-success.ogt", Sound.Welcome, "ui", 0.6],
    ["audio/ui-press.ogt", Sound.Move, "ui", 0.3],
  ];
  for (const [path, bit, bus, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus, volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/orbit.wav"), bus: "music", loop: true, volume: 0.55, priority: 220 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 4, duckRelease: 0.6 }));
  const game = must(world.resource(Game));
  game.orbY = 0.35;
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `play`: from the title to the orb, as Play does. */
function play(world: World): void {
  gameOf(world).phase = Phase.Play;
}

/** `sparks:<n>`: set the sparks (and count them as made this run). */
function sparks(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const n = num(args[0], "n");
  const more = Math.max(0, n - g.sparks);
  g.sparks = n;
  g.runEarned += more;
  g.lifetime += more;
}

/** `own:<generator 0-3>:<n>`: set how many of a generator are owned. */
function own(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const i = num(args[0], "generator");
  if (i < 0 || i > 3) throw new Error("the generator is 0 to 3");
  setOwned(g, i, num(args[1], "n"));
}

/** `supernova`: collapse now, as the dialog's Collapse does (refused under one stardust). */
function supernova(world: World): void {
  nova(world, gameOf(world), lazyTuning(world));
}

/**
 * `relaunch:<seconds away>`: the game closed and opened again that much later, in this process:
 * the run forgotten, the save read again, the time away credited. For tests: a real launch reads
 * the time away from the host's clock (the save's `at` against its answer's time; tests/away.og.ts
 * runs it across two processes).
 */
function relaunch(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const away = num(args[0], "seconds");
  g.phase = Phase.Title;
  g.paused = false;
  g.settingsOpen = false;
  g.confirmNova = false;
  g.welcome = false;
  g.sparks = 0;
  g.runEarned = 0;
  g.lifetime = 0;
  g.owned0 = 0;
  g.owned1 = 0;
  g.owned2 = 0;
  g.owned3 = 0;
  g.dust = 0;
  g.novas = 0;
  g.runTicks = 0;
  g.taps = 0;
  g.rate = 0;
  g.offline = 0;
  g.loaded = false;
  g.saveRevision = 0;
  g.away = away;
  launch(world, g, must(world.resource(Prefs)));
}

/** `stale_save`: another device writes a save with more lifetime sparks, so this one's next write is refused as stale and the other's progress kept. */
function staleSave(world: World): void {
  must(og.platform.storeSet(world, { key: "orb", value: { v: 1, sparks: 5000, runEarned: 9000, lifetime: 1e9, owned: [20, 5, 0, 0], dust: 3, novas: 1, runTicks: 600, taps: 10 } }));
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

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning, Prefs],
  systems: [Ui, OrbTap, Economy, Motes, Bursts, Voices],
  render: { antialias: "off" },
  look: "looks/idle.ogt",
  start,
  commands: { play, sparks, own, supernova, relaunch, stale_save: staleSave, viewport, language },
  commandHelp: {
    play: { usage: "", doc: "From the title to the orb, as Play does." },
    sparks: { usage: "<n>", doc: "Set the sparks (the rise counts as made this run)." },
    own: { usage: "<generator 0-3>:<n>", doc: "Set how many of a generator are owned." },
    supernova: { usage: "", doc: "Collapse now, as the dialog's Collapse does (refused under one stardust)." },
    relaunch: { usage: "<seconds away>", doc: "Close and reopen the game that much later in this process (for tests; a real launch reads the time away from the host's clock): the save read again, the time away credited." },
    stale_save: { usage: "", doc: "Another device writes a save with more progress, so this one's next write is refused as stale and merged." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "nova"][g.phase],
      sparks: g.sparks,
      rate: g.rate,
      tapValue: g.tapValue,
      owned: counts(g),
      dust: g.dust,
      novas: g.novas,
      runEarned: g.runEarned,
      lifetime: g.lifetime,
      taps: g.taps,
      buys: g.buys,
      refused: g.refused,
      saves: g.saves,
      saveConflicts: g.saveConflicts,
      offline: g.offline,
      language: LANGUAGES[g.language]!.code,
    };
  },
});

