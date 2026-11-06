// Greywatch Keep, the rpg archetype (3D profile): the dead have risen in
// the yard of a border keep. Castellan Maren waits by the gate; talk to her
// (a branching og.story conversation, assets/story/maren.ogt) and take her
// quest (assets/story/seal.ogt): put down the four risen and their captain,
// take the warden's seal from the chest by the crypt door, and bring it
// back to her. The risen walk the keep's navmesh (og.nav) and think with
// og.ai machines (assets/ai/): they see you, hunt you and wind up their
// blows. Strike back with the sword (a hit stop, a flash, the numbers, the
// sparks), drink a draught when it goes badly, pick up what they drop, and
// level up on what you put down. Handing Maren the seal wins the run; the
// party falling ends it. Then the summary, and again.
//
// Every hero is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the keep plays alone, two on one device as a party
// (the knight and the barbarian, the screen split in halves, each half
// its hero's own camera: src/view.ts) or two over a room in lockstep (each
// peer's camera on its own hero), as src/creation.ts says. The party
// shares the bag (gold and draughts), the quest and the experience; a
// fallen hero rises at the gate while the other stands.
//
// The view is creation.ts's: isometric (the default; the look turns the
// hero and the stick walks the screen's way), third person behind each
// hero, or first person at its eye (src/view.ts).
//
// `start` loads the strings, the style, the sounds, the screen documents
// (assets/ui/) and the story, builds the keep (src/world.ts) and spawns
// the heroes (src/heroes.ts), Maren, the risen (src/foes.ts) and the
// voices (src/sound.ts). The screens are src/ui.ts, the conversation and
// the quest src/story.ts, the state src/components.ts.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { animate, rigOf } from "./anim.ts";
import { Foe, Game, Hero, Phase, Tuning } from "./components.ts";
import { prefs } from "./device.ts";
import { alarm, Foes, slay, spawnFoes, strike } from "./foes.ts";
import { BODY_SCALE, entity, EYE, FEET, heroOf, Heroes, hurtHero, layerCameras, spawnHeroes } from "./heroes.ts";
import { FOES, MAREN } from "./layout.ts";
import { LOCAL, SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { loadStory, QUEST, startTalk, Story } from "./story.ts";
import { LANGUAGES, setLanguage, startRun, toTitle, Ui } from "./ui.ts";
import { ISOMETRIC, SeatLook, setupView, ViewRig } from "./view.ts";
import { buildKeep, flicker, must } from "./world.ts";

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/greywatch/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));
  loadStory(world);
  buildKeep(world);
  const heroes = spawnHeroes(world);
  // Third person sits 4.6 m behind and a metre over the hero's middle, raised as the look tips down.
  setupView(world, heroes, { eye: EYE, behind: 4.6, above: 1.0, isoHeight: ISO_HEIGHT });
  layerCameras(world);
  const cam = must(world.resource(og.Camera));
  playCamera = { orthographic: cam.orthographic, height: cam.height, fov: cam.fov, layers: cam.layers };
  spawnMaren(world);
  spawnFoes(world);
  const sparks = must(world.spawn({ name: "sparks", position: { x: 0, y: -5, z: 0 } }));
  must(world.insert(sparks, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/sparks.ogt")), seed: 21, visible: false }));
  spawnVoices(world);
  // Behind the title the yard is reset, its torches burning.
  toTitle(world, gameOf(world));
}

/** Castellan Maren by the gate (the KayKit mage), idle, turning to whoever talks to her. */
function spawnMaren(world: World): void {
  const g = gameOf(world);
  const rig = rigOf(world, "mage");
  const e = must(world.spawn({ name: "maren", position: { x: MAREN.x, y: 0, z: MAREN.z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, MAREN.yaw) }));
  const body = must(world.instantiate(must(og.load.gltf(world, "models/mage.glb")), { name: "maren-body", parent: e, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
  animate(world, body, rig);
  g.maren = e as unknown as number;
  // The quest's mark over her head while she has something to say.
  const glow = must(world.spawn({ name: "chest-glow", position: { x: 0, y: 0.4, z: -10.8 } }));
  must(world.insert(glow, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/levelup.ogt")), seed: 5, visible: false }));
}

/** The isometric view's height in metres in a run, and over the title (the whole yard). */
const ISO_HEIGHT = 15;
const TITLE_HEIGHT = 27;
/** The main camera as the view set it up for a run (src/view.ts), given back when the title leaves. */
let playCamera = { orthographic: false, height: 0, fov: 55, layers: 1 };

/**
 * Behind the title the camera turns slowly round the yard, the whole keep
 * in view (orthographic on the isometric view, at its fixed pitch); in a
 * run the view's controllers (src/view.ts) have it, and with two seats on
 * one device the halves draw instead. Listed after the camera systems, so
 * the title's orbit is the last word.
 */
const TitleCamera = defineSystem("TitleCamera", { reads: [Game], writes: [og.Camera, og.CameraView] }, (_q, time, world) => {
  const g = gameOf(world);
  const cam = must(world.resource(og.Camera));
  const title = g.phase === Phase.Title;
  if (LOCAL) for (let seat = 0; seat < SEATS; seat++) must(world.set(entity(world, `view-${seat}`), og.CameraView, { active: !title }));
  if (!title) {
    if (cam.layers !== playCamera.layers || cam.orthographic !== playCamera.orthographic || cam.height !== playCamera.height || cam.fov !== playCamera.fov) Object.assign(cam, playCamera);
    return;
  }
  const a = Math.PI / 4 + 0.35 * math.sin(time.elapsed * 0.05);
  const pitch = ISOMETRIC ? (35.264 * Math.PI) / 180 : 0.5;
  const r = ISOMETRIC ? 60 : 30;
  const focus = { x: 0, y: 0, z: -0.5 };
  const eye = { x: focus.x + r * math.cos(pitch) * math.sin(a), y: r * math.sin(pitch), z: focus.z + r * math.cos(pitch) * math.cos(a) };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, focus);
  cam.orthographic = ISOMETRIC;
  cam.height = TITLE_HEIGHT;
  cam.fov = 50;
  cam.layers = 1;
});

/** Maren turns to the hero who talks to her, and back to the yard after; the torches flicker. */
const Keep = defineSystem("Keep", { reads: [Game, Hero], writes: [og.Transform, og.PointLight], after: ["Story"] }, (_q, time, world) => {
  const g = gameOf(world);
  flicker(world, time.elapsed);
  const maren = g.maren as unknown as Entity;
  let yaw = MAREN.yaw;
  if (g.talker >= 0) {
    const p = must(world.get(heroOf(g, g.talker), og.Transform)).position;
    yaw = math.atan2(p.x - MAREN.x, p.z - MAREN.z);
  }
  must(world.set(maren, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) }));
});

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

function seatOf(s: string | undefined): number {
  const seat = num(s, "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
  return seat;
}

/** Where a command's place names: maren, chest, gate, foe<i>, loot<i>, or x:z. */
function placeOf(world: World, args: readonly string[]): { x: number; z: number } {
  const name = args[0] ?? "";
  if (name === "maren") return { x: MAREN.x + 1.4, z: MAREN.z + 0.4 };
  if (name === "chest") return { x: 0, z: -9.5 };
  if (name === "gate") return { x: -7.2, z: 9.4 };
  if (name.startsWith("loot")) {
    const p = must(world.get(entity(world, `loot-${num(name.slice(4), "loot")}`), og.Transform)).position;
    return { x: p.x, z: p.z };
  }
  if (name.startsWith("foe")) {
    const p = must(world.get(entity(world, `foe-${num(name.slice(3), "foe")}`), og.Transform)).position;
    return { x: p.x + 1.5, z: p.z + 0.8 };
  }
  return { x: num(args[0], "x"), z: num(args[1], "z") };
}

/** `run`: start a run now, as Enter the keep does. */
function run(world: World): void {
  startRun(world, gameOf(world));
}

/** `warp:<place>`: put seat 0's hero by Maren, the chest, the gate or a risen (foe0 to foe4), facing it, on a loot slot (loot0 to loot7), or at x:z. */
function warp(world: World, args: readonly string[]): void {
  warpSeat(world, 0, args);
}

/** `warp_seat:<seat>:<place>`: the same for any seat's hero. */
function warpOf(world: World, args: readonly string[]): void {
  warpSeat(world, seatOf(args[0]), args.slice(1));
}

function warpSeat(world: World, seat: number, args: readonly string[]): void {
  const g = gameOf(world);
  const hero = heroOf(g, seat);
  const at = placeOf(world, args);
  must(world.set(hero, og.Transform, { position: { x: at.x, y: FEET + 0.05, z: at.z } }));
  const name = args[0] ?? "";
  const to = name === "maren" ? { x: MAREN.x, z: MAREN.z } : name === "chest" ? { x: 0, z: -10.8 } : name.startsWith("foe") ? { x: at.x - 1.5, z: at.z - 0.8 } : null;
  if (to === null) return;
  // It faces the place, and its look with it (the look's heading is the facing turned half round).
  const facing = math.atan2(to.x - at.x, to.z - at.z);
  must(world.set(hero, Hero, { facing }));
  must(world.set(hero, SeatLook, { yaw: facing + (facing > 0 ? -Math.PI : Math.PI) }));
}

/** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back): the room tests. */
function bot(_world: World, args: readonly string[]): void {
  setBot(num(args[0], "n"));
}

/** `talk`: seat 0's hero starts talking to Maren wherever it stands. */
function talk(world: World): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play || g.conversation !== 0) throw new Error("talk needs a run and no conversation (write run first)");
  startTalk(world, g, 0);
}

/** `quest:<stage>`: the quest given (clear), the yard cleared (fetch: every risen slain) or the seal taken (report). */
function quest(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("quest needs a run (write run first)");
  const stage = args[0] ?? "";
  if (!["clear", "fetch", "report"].includes(stage)) throw new Error("write quest:<clear|fetch|report>");
  must(og.story.startQuest(world, QUEST));
  if (stage === "clear") return;
  FOES.forEach((_, i) => slay(world, [`${i}`]));
  if (stage === "report") {
    og.story.progress(world, QUEST, "seal");
    g.seal = true;
    g.chestOpen = true;
  }
}

/** `hp:<seat>:<n>`: set a hero's health (0 fells it, as a risen's blow would). */
function hp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const hero = heroOf(g, seatOf(args[0]));
  const me = must(world.get(hero, Hero));
  const n = num(args[1], "n");
  if (n > 0) {
    must(world.set(hero, Hero, { hp: Math.min(me.maxHp, n) }));
    return;
  }
  // The fall itself is a blow's (src/heroes.ts hurtHero), from a step to the east.
  must(world.set(hero, Hero, { invulnerable: 0 }));
  const p = must(world.get(hero, og.Transform)).position;
  hurtHero(world, g, must(world.resource(Tuning)), hero, me.hp + 1, { x: p.x + 1, z: p.z });
}

/** `potions:<n>`: set the party's draughts. */
function potions(world: World, args: readonly string[]): void {
  gameOf(world).potions = num(args[0], "n");
}

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  setLanguage(world, prefs, at);
}

/** `stale_record`: another device writes the record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { runs: 9, wins: 3, best: 200 } }));
}

const r2 = (n: number) => Math.round(n * 100) / 100;

/** A seat's hero, for the facts. */
function heroFacts(world: World, g: ReturnType<typeof gameOf>, seat: number, suffix: string) {
  const hero = heroOf(g, seat);
  const me = world.get(hero, Hero);
  const at = world.get(hero, og.Transform);
  if (!me.ok || !at.ok) return {};
  const m = me.value;
  return {
    [`hp${suffix}`]: Math.round(m.hp), [`maxHp${suffix}`]: Math.round(m.maxHp), [`level${suffix}`]: m.level, [`xp${suffix}`]: Math.round(m.xp),
    [`dead${suffix}`]: m.dead, [`near${suffix}`]: ["none", "maren", "chest"][m.near], [`speed${suffix}`]: r2(m.speed), [`facing${suffix}`]: r2(m.facing),
    [`swings${suffix}`]: m.swings, [`kills${suffix}`]: m.kills, [`x${suffix}`]: r2(at.value.position.x), [`z${suffix}`]: r2(at.value.position.z),
  };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Seats, ViewRig, ...og.systems.cameras, TitleCamera, Ui, Heroes, Foes, Story, Keep, Voices],
  start,
  commands: { run, warp, warp_seat: warpOf, bot, talk, quest, slay, hp, potions, viewport, language, stale_record: staleRecord, alarm, strike },
  commandHelp: {
    run: { usage: "", doc: "Start a run now, as Enter the keep does." },
    warp: { usage: "<maren|chest|gate|foe<i>|loot<i>> | <x>:<z>", doc: "Put seat 0's hero by Maren, the chest, the gate or a risen (foe0 to foe4), facing it, on a loot slot (loot0 to loot7), or at x, z." },
    warp_seat: { usage: "<seat>:<place> | <seat>:<x>:<z>", doc: "The same for any seat's hero." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots play the same run." },
    talk: { usage: "", doc: "Seat 0's hero starts talking to Maren wherever it stands." },
    quest: { usage: "<clear|fetch|report>", doc: "The quest given (clear), every risen slain (fetch) or the seal in the bag too (report): hand it to Maren to win." },
    slay: { usage: "<risen>", doc: "A risen (0 to 4; 4 is the captain) falls now, as to seat 0's blow: the quest counts it, the party gains its experience, it drops its loot." },
    hp: { usage: "<seat>:<n>", doc: "Set a hero's health; 0 fells it as a risen's blow would." },
    potions: { usage: "<n>", doc: "Set the party's draughts." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the record, so this one's next write is refused as stale and merged." },
    alarm: { usage: "<risen>", doc: "A risen (0 to 4, or its entity: og.ai sends it) has seen a hero (its behaviour raises it entering the hunt)." },
    strike: { usage: "<risen>", doc: "A risen (0 to 4, or its entity) winds up a blow at the nearest hero (its behaviour raises it within reach)." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    const v = g.conversation !== 0 ? og.story.view(world, g.conversation) : null;
    const foes = FOES.map((_, i) => world.get(entity(world, `foe-${i}`), Foe));
    return {
      phase: ["title", "play", "over"][g.phase],
      ending: ["none", "won", "fallen"][g.ending],
      paused: g.paused,
      quest: ["none", "clear", "fetch", "return", "done"][g.quest],
      slain: g.slain,
      standing: foes.filter((f) => f.ok && f.value.hp > 0).length,
      alert: foes.filter((f) => f.ok && f.value.alert).length,
      strikes: foes.reduce((n, f) => n + (f.ok ? f.value.strikes : 0), 0),
      gold: g.gold,
      potions: g.potions,
      seal: g.seal,
      chest: g.chestOpen,
      talking: g.conversation !== 0,
      node: v?.node ?? "",
      line: v?.key ?? "",
      replies: v?.choices.length ?? 0,
      hits: g.hits,
      hurts: g.hurts,
      drunk: g.drunk,
      levelUps: g.levelUps,
      time: Math.round(g.time * 10) / 10,
      runs: g.runs,
      wins: g.wins,
      language: LANGUAGES[prefs.language]!.code,
      seats: SEATS,
      ...heroFacts(world, g, 0, ""),
      ...(SEATS > 1 ? heroFacts(world, g, 1, "1") : {}),
    };
  },
});
