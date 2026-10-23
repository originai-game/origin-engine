// Path Guard, the tower-defense archetype (2D profile): a night garden,
// visitors coming through the gate in the hedge and walking to the
// lantern on the doorstep, and the plants a player sets on the lawn to
// turn them away. Thorns shoot, frost lilies slow, hedges only block, and
// every plant re-routes the visitors: they walk og.nav's flow field to the
// doorstep, which goes round whatever stands on the lawn. Five waves; the
// lantern's light runs out after ten visitors get through.
//
// `start` loads the field (assets/maps/garden.ogt: a tilemap whose hedges
// and stones block), makes the walking grid and the flow field from it,
// and spawns the pools: one Bed row per lawn cell (off until a plant is
// set), the visitors, the shots, the cursor, the lantern and one voice per
// sound. The systems are in src/systems.ts, the screens in src/ui.ts, the
// state in src/components.ts.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { checkClips, clipOf } from "./art.ts";
import { Bed, Cursor, Game, Phase, Prefs, Sfx, Shot, Sound, Tuning, Visitor } from "./components.ts";
import { buildable, cellCenter, COLS, Control, DOOR, endRound, fire, place, Plants, ROWS, SHOTS, Visitors, VISITORS, Voices, WAVES } from "./systems.ts";
import { LANGUAGES, startRound, Ui } from "./ui.ts";

/** The night behind the field, linear. */
const NIGHT = { r: 0.0047, g: 0.0137, b: 0.0091 };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
function must(r: { ok: true } | { ok: false; reason: string }): void;
function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function start(world: World): void {
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/pathguard/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(
    og.ui.configure(world, {
      style,
      sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") },
    }),
  );

  const settings = must(world.resource(og.RenderSettings));
  settings.environmentIntensity = 0;
  settings.toneMapped = false;
  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = 14;
  camera.position = { x: COLS / 2, y: ROWS / 2, z: 50 };
  camera.background = NIGHT;

  const sheet = must(og.load.flipbook(world, "sprites/garden.ogt"));
  checkClips(sheet);
  const tilemap = must(og.load.tilemap(world, "maps/garden.ogt"));
  const map = must(world.spawn({ name: "garden", position: { x: 0, y: tilemap.height * tilemap.tileHeight, z: 0 } }));
  must(world.insert(map, og.Tilemap, { map: tilemap.map }));
  // The walking grid from the hedges and stones, and the flow field every visitor follows to the doorstep.
  const grid = must(og.nav.gridFromTilemap(world, { entity: map, layers: ["walls"] }));
  const goal = cellCenter(DOOR.x, DOOR.y);
  const field = must(og.nav.flowField(world, { grid, targets: [{ x: goal.x, y: goal.y, z: 0 }] }));
  const mind = must(og.ai.load(world, "ai/plant.ogt"));
  const pop = must(og.load.emitter(world, "fx/pop.ogt"));

  const game = must(world.resource(Game));
  game.grid = grid as unknown as number;
  game.field = field as unknown as number;
  game.map = map as unknown as number;
  game.sheet = sheet.texture as unknown as number;
  game.ring = must(og.load.texture(world, "sprites/ring.png")) as unknown as number;

  const sprite = (e: Entity, clip: string, layer: number, visible: boolean, size = 1) => {
    must(world.insert(e, og.Sprite, { texture: sheet.texture, width: size, height: size, layer, visible, sortY: true }));
    must(world.insert(e, og.Flipbook, { sheet: sheet.sheet, clip: clipOf(clip) }));
  };

  // One bed per lawn cell: a plant there is this row switched on.
  for (let y = 1; y < ROWS - 1; y++) {
    for (let x = 1; x < COLS - 1; x++) {
      const c = cellCenter(x, y);
      const bed = must(world.spawn({ name: `bed-${x}-${y}`, position: { x: c.x, y: c.y, z: 0 } }));
      sprite(bed, "thorn", 1, false);
      must(world.insert(bed, Bed, { cellX: x, cellY: y }));
      must(world.insert(bed, og.AiAgent, { behaviour: mind.behaviour, active: false }));
      must(world.insert(bed, og.Perception, { sight: 0, fov: 360, occlude: false, team: 1, memory: 0 }));
    }
  }
  for (let i = 0; i < VISITORS; i++) {
    const v = must(world.spawn({ name: `visitor-${i}`, position: { x: -40, y: -40, z: 0 } }));
    sprite(v, "beetle", 2, false, 0.9);
    must(world.insert(v, Visitor, { slot: i }));
    must(world.insert(v, og.NavAgent, { nav: field, target: { x: goal.x, y: goal.y, z: 0 }, seeking: false, speed: 1, acceleration: 30, radius: 0.3, stopDistance: 0.2, slowRadius: 0, faceMovement: true, turnSpeed: 14 }));
    must(world.insert(v, og.Perceivable, { team: 2 }));
    must(world.insert(v, og.ParticleSystem, { emitter: pop, seed: i + 1, visible: false }));
  }
  for (let i = 0; i < SHOTS; i++) {
    const s = must(world.spawn({ name: `shot-${i}`, position: { x: -40, y: -40, z: 0 } }));
    sprite(s, "seed", 5, true, 0.6);
    must(world.set(s, og.Sprite, { opacity: 0, sortY: false }));
    must(world.insert(s, Shot));
  }
  const lantern = must(world.spawn({ name: "lantern", position: { x: goal.x, y: goal.y + 0.1, z: 0 } }));
  sprite(lantern, "lantern", 3, true, 1.1);
  // The night: a pale glow behind the field and fireflies over it.
  const glow = must(world.spawn({ name: "moonlight", position: { x: COLS / 2, y: ROWS / 2, z: 0 } }));
  must(world.insert(glow, og.Sprite, { texture: must(og.load.texture(world, "sprites/glow.png")), width: 24, height: 24, color: { r: 0.45, g: 0.75, b: 0.55 }, opacity: 0.22, layer: -40 }));
  const flies = must(world.spawn({ name: "fireflies", position: { x: COLS / 2, y: ROWS / 2, z: 0 } }));
  must(world.insert(flies, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/fireflies.ogt")), seed: 7 }));
  const cursor = must(world.spawn({ name: "cursor", position: { x: -40, y: -40, z: 0 } }));
  must(world.insert(cursor, Cursor));

  // The voices: one per sound, played by Voices; the music loops on its bus.
  const voices: [string, number, "sfx" | "ui", number][] = [
    ["audio/plant.wav", Sound.Plant, "ui", 0.7],
    ["audio/thorn.wav", Sound.Thorn, "sfx", 0.35],
    ["audio/frost.wav", Sound.Frost, "sfx", 0.35],
    ["audio/pop.wav", Sound.Pop, "sfx", 0.6],
    ["audio/leak.wav", Sound.Leak, "sfx", 0.8],
    ["audio/wave.wav", Sound.Wave, "sfx", 0.6],
    ["audio/clear.wav", Sound.Clear, "sfx", 0.6],
    ["audio/won.wav", Sound.Won, "sfx", 0.7],
    ["audio/lost.wav", Sound.Lost, "sfx", 0.7],
    ["audio/ui-press.ogt", Sound.Move, "ui", 0.25],
  ];
  for (const [path, bit, bus, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus, volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/night.wav"), bus: "music", loop: true, volume: 0.55, priority: 220 }));
  // The music ducks under the hits and the waves (og-audio §4).
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 6, duckRelease: 0.5 }));
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `begin`: start a round now, as Play on the title does. */
function begin(world: World): void {
  startRound(world, gameOf(world), must(world.resource(Tuning)));
}

/** `plant:<thorn|frost|hedge>:<x>:<y>`: set a plant on a map cell (column, row from the top) as a placement does, refusals included. */
function plantCmd(world: World, args: readonly string[]): void {
  const kind = ["thorn", "frost", "hedge"].indexOf(args[0] ?? "");
  if (kind < 0) throw new Error("write plant:<thorn|frost|hedge>:<x>:<y>");
  const g = gameOf(world);
  g.selected = kind;
  place(world, g, must(world.resource(Tuning)), num(args[1], "x"), num(args[2], "y"));
}

/** `gold:<n>`: set the purse. */
function gold(world: World, args: readonly string[]): void {
  gameOf(world).gold = num(args[0], "n");
}

/** `wave:<n>`: skip to the break before wave n (the field and the purse stay). */
function wave(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const n = Math.max(1, Math.min(WAVES.length, num(args[0], "n")));
  g.wave = n - 1;
  g.inWave = false;
  g.toSpawn = 0;
  g.breakTicks = Math.max(1, Math.round(must(world.resource(Tuning)).waveBreak * 60));
}

/** `lose`: put the lantern out (the round ends lost on the next tick). */
function lose(world: World): void {
  const g = gameOf(world);
  if (g.phase === Phase.Play) g.lives = 0;
}

/** `win`: end the round as won now. */
function win(world: World): void {
  const g = gameOf(world);
  if (g.phase === Phase.Play) endRound(world, g, true);
}

/** `viewport:<desktop|phone>`: lay the UI out for that surface (og.ui.previewViewport). */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`: switch the language as the settings do. */
function language(world: World, args: readonly string[]): void {
  const p = must(world.resource(Prefs));
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error(`write language:<${LANGUAGES.map((l) => l.code).join("|")}>`);
  p.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning, Prefs],
  systems: [Control, Plants, Visitors, Ui, Voices],
  render: { antialias: "off" },
  start,
  commands: { fire, begin, plant: plantCmd, gold, wave, lose, win, viewport, language },
  commandHelp: {
    fire: { usage: "<plant>", doc: "The plants' behaviour command (og.ai queues it with the plant's handle): a shot at what it sees." },
    begin: { usage: "", doc: "Start a round now, as Play on the title does." },
    plant: { usage: "<thorn|frost|hedge>:<x>:<y>", doc: "Set a plant on a map cell (column, row from the top) as a placement does, refusals included." },
    gold: { usage: "<n>", doc: "Set the purse." },
    wave: { usage: "<n>", doc: "Skip to the break before wave n (the field and the purse stay)." },
    lose: { usage: "", doc: "Put the lantern out: the round ends lost." },
    win: { usage: "", doc: "End the round as won now." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×) or back for the surface." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "over"][g.phase],
      paused: g.paused,
      settings: g.settingsOpen,
      wave: g.wave,
      waves: WAVES.length,
      inWave: g.inWave,
      lives: g.lives,
      gold: g.gold,
      kills: g.kills,
      leaks: g.leaks,
      placed: g.placed,
      refused: g.refused,
      refusal: ["none", "poor", "blocked", "taken"][g.refusal],
      shots: g.shots,
      walking: g.walking,
      selected: ["thorn", "frost", "hedge"][g.selected],
      cursor: { x: g.cursorX, y: g.cursorY },
      cursorBuildable: buildable(world, g, g.cursorX, g.cursorY),
      won: g.won,
      score: g.score,
      best: g.best,
      newBest: g.newBest,
      rounds: g.rounds,
      language: LANGUAGES[g.language]!.code,
    };
  },
});
