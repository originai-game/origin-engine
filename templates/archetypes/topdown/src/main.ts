// Tomb Room, the topdown archetype (2D profile): a small Zelda-like. Two
// rooms of a tomb, a slime guarding the key in the west room, coins in
// both, and a locked doorway between them.
//
// Everything is placed by the Tiled map assets/maps/tomb.tmj: its tile
// layers (floor, the wall terrain that auto-tiles and collides, the door,
// torches drawn above the hero) and its objects (the spawn, the enemy, the
// pickups, the door and the two rooms). `start` below loads it, spawns the
// actors at its objects, gives each room a camera zone and the hero a 2D
// follow camera. The slime's mind is data: assets/ai/slime.ogt, a behaviour
// tree og.ai runs (patrol, chase on sight, bite, flee when hurt), walking a
// grid og.nav makes from the map's wall collision. The systems are in
// src/systems.ts, the state in src/components.ts, the flipbook sheets in
// src/art.ts.

import { og, BODY_DYNAMIC, LOCK_ROTATION_X, LOCK_ROTATION_Y, LOCK_ROTATION_Z, LOCK_TRANSLATION_Z, SHAPE_BOX } from "@origingame/engine";
import type { Entity, FlipbookInfo, World } from "@origingame/engine";
import { checkHeroClips, clip, heroClip, HeroClip, loadSheets } from "./art.ts";
import { Facing, Game, Hero, Pickup, PickupKind, Report, Slime, Sfx, Sound, Tuning } from "./components.ts";
import { biteHero, Control, Door, Hud, openDoor, Pickups, Slimes, Voices } from "./systems.ts";

/** The visible height in tiles: a whole room (12 tall), as in the old top-down adventures. */
const VIEW_HEIGHT = 12;
/** The brand ink #14161b, linear. */
const INK = { r: 0.00699, g: 0.00802, b: 0.01096 };
/** Sprite layers: the map's floor draws at -10, its walls at -9, the door at -8, torches at +5. */
const ACTOR_LAYER = 0;

/** Collision layers: the map's walls are layer 1; the hero and slimes collide with walls only, not each other. Walls also hide the hero from a slime. */
const WALLS = 1;
const HERO = 4;
const SLIMES = 2;

const LOCK_2D = LOCK_TRANSLATION_Z | LOCK_ROTATION_X | LOCK_ROTATION_Y | LOCK_ROTATION_Z;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
function must(r: { ok: true } | { ok: false; reason: string }): void;
function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A sorted, shadowed sprite one tile big with its feet at the entity, playing a flipbook clip. */
function actor(world: World, name: string, x: number, y: number, sheet: FlipbookInfo, clipIndex: number, shadow: number): Entity {
  const e = must(world.spawn({ name, position: { x, y, z: 0 } }));
  must(
    world.insert(e, og.Sprite, {
      width: 1,
      height: 1,
      anchor: { x: 0.5, y: 0 },
      layer: ACTOR_LAYER,
      sortY: true,
      shadow,
      shadowSize: { x: 0.8, y: 0.28 },
      shadowOffset: { x: 0, y: 0.04 },
    }),
  );
  must(world.insert(e, og.Flipbook, { sheet: sheet.sheet, clip: clipIndex }));
  return e;
}

/** A dynamic body that slides along the walls: no gravity, no turning, a box at the feet. */
function body(world: World, e: Entity, w: number, h: number, layer: number): void {
  must(world.insert(e, og.RigidBody, { kind: BODY_DYNAMIC, gravityScale: 0, lock: LOCK_2D, linearDamping: 0 }));
  must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: h, z: 1 }, offset: { x: 0, y: h / 2, z: 0 }, friction: 0, layers: layer, mask: WALLS }));
  must(world.insert(e, og.Velocity));
}

function start(world: World): void {
  const sheets = loadSheets(world);
  checkHeroClips(sheets.hero);
  const font = must(og.load.font(world, "fonts/ui.ttf"));
  const tilemap = must(og.load.tilemap(world, "maps/tomb.tmj"));

  // The map's top-left corner at (0, height): world y runs up from the map's bottom edge at 0.
  const top = tilemap.height * tilemap.tileHeight;
  const map = must(world.spawn({ name: "tomb", position: { x: 0, y: top, z: 0 } }));
  must(world.insert(map, og.Tilemap, { map: tilemap.map }));
  // The slimes' walking grid: a cell per tile, blocked where the walls collide.
  const grid = must(og.nav.gridFromTilemap(world, { entity: map }));
  const mind = must(og.ai.load(world, "ai/slime.ogt"));

  const settings = must(world.resource(og.RenderSettings));
  settings.environmentIntensity = 0;
  settings.toneMapped = false;
  const physics = must(world.resource(og.Physics));
  physics.gravity = { x: 0, y: 0, z: 0 };

  const game = must(world.resource(Game));
  game.font = font as unknown as number;
  let spawn = { x: 2, y: 2 };
  let coins = 0;
  let split = tilemap.width * tilemap.tileWidth;

  for (const o of tilemap.objects) {
    const x = o.position.x;
    const yTop = top + o.position.y;
    const cx = o.center.x;
    const cy = top + o.center.y;
    switch (o.kind) {
      case "spawn":
        spawn = { x, y: yTop };
        break;
      case "enemy": {
        const s = actor(world, o.name, x, yTop, sheets.slime, clip(sheets.slime, "bob"), 0.4);
        body(world, s, 0.7, 0.4, SLIMES);
        must(world.insert(s, Slime, { bob: clip(sheets.slime, "bob"), hurt: clip(sheets.slime, "hurt") }));
        // og.nav walks it (writing its velocity), og.ai decides where; it sees the hero all round, not through walls.
        must(world.insert(s, og.NavAgent, { nav: grid, speed: 1.6, acceleration: 30, radius: 0.35, stopDistance: 0.1, repathTicks: 20 }));
        must(world.insert(s, og.AiAgent, { behaviour: mind.behaviour }));
        must(world.insert(s, og.Perception, { sight: 5, fov: 360, eyeHeight: 0, occluders: WALLS, team: 2, memory: 3 }));
        game.enemyHp = must(world.get(s, Slime)).hp;
        break;
      }
      case "coin":
      case "key": {
        const key = o.kind === "key";
        const p = actor(world, o.name, x, yTop, sheets.items, clip(sheets.items, key ? "key" : "coin"), 0.3);
        must(world.insert(p, Pickup, { kind: key ? PickupKind.Key : PickupKind.Coin, baseY: yTop }));
        if (!key) coins += 1;
        break;
      }
      case "door":
        game.doorX = cx;
        game.doorY = cy;
        game.doorCellX = Math.round(o.position.x / tilemap.tileWidth);
        game.doorCellY = Math.round(-o.position.y / tilemap.tileHeight);
        game.doorCellsW = Math.round(o.size.x / tilemap.tileWidth);
        game.doorCellsH = Math.round(o.size.y / tilemap.tileHeight);
        break;
      case "room": {
        // A camera zone per room: the view stays inside the room the hero stands in.
        const z = must(world.spawn({ name: o.name }));
        must(world.insert(z, og.CameraZone, { min: { x, y: yTop - o.size.y }, max: { x: x + o.size.x, y: yTop } }));
        if (x > 0) split = Math.min(split, x);
        break;
      }
    }
  }
  game.coinsTotal = coins;
  game.roomSplit = split;
  game.spawnX = spawn.x;
  game.spawnY = spawn.y;
  game.heroX = spawn.x;
  game.heroY = spawn.y;

  // The hero: the sprite, the walk, the body, the dash's ghosts and the camera.
  const hero = actor(world, "hero", spawn.x, spawn.y, sheets.hero, heroClip(HeroClip.Idle, Facing.Down), 0.35);
  body(world, hero, 0.6, 0.35, HERO);
  must(world.insert(hero, Hero));
  must(world.insert(hero, Report));
  must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0 }));
  must(world.insert(hero, og.Afterimage, { count: 4, interval: 2, opacity: 0.5, color: { r: 1, g: 0.75, b: 0.45 }, active: false }));
  must(world.insert(hero, og.Follow2D, { offset: { x: 0, y: 0.5 }, deadZone: { x: 0.6, y: 0.4 }, smoothTime: 0.12, lookAhead: 0.15, lookAheadMax: { x: 1.5, y: 1 } }));

  // An orthographic camera on the spawn's room.
  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = VIEW_HEIGHT;
  camera.position = { x: spawn.x, y: spawn.y + 0.5, z: 50 };
  camera.background = INK;

  // The voices: one per sound, played by the Voices system.
  const voices: [string, number][] = [
    ["audio/hit.wav", Sound.Hit],
    ["audio/hurt.wav", Sound.Hurt],
    ["audio/coin.wav", Sound.Coin],
    ["audio/key.wav", Sound.Key],
    ["audio/door.wav", Sound.Door],
  ];
  for (const [path, bit] of voices) {
    const sound = must(og.load.sound(world, path));
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound, playing: false, volume: 0.6 }));
    must(world.insert(v, Sfx, { bit }));
  }
}

function heroOf(world: World): Entity {
  const hero = world.find("hero");
  if (hero === null) throw new Error("no hero yet");
  return hero;
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `place:<x>:<y>`: put the hero's feet at a world point, standing still. */
function place(world: World, args: readonly string[]): void {
  const hero = heroOf(world);
  const x = num(args[0], "x");
  const y = num(args[1], "y");
  must(world.set(hero, og.Transform, { position: { x, y, z: 0 } }));
  must(world.set(hero, og.Velocity, { linear: { x: 0, y: 0, z: 0 } }));
  const g = must(world.resource(Game));
  g.heroX = x;
  g.heroY = y;
}

/** `slime:<x>:<y>`: move the slime there. */
function slime(world: World, args: readonly string[]): void {
  const s = world.find("slime");
  if (s === null) throw new Error("the map has no slime");
  must(world.set(s, og.Transform, { position: { x: num(args[0], "x"), y: num(args[1], "y"), z: 0 } }));
}

/** `give:<key|coin|door>`: the key, a coin, or the doorway opened. */
function give(world: World, args: readonly string[]): void {
  const g = must(world.resource(Game));
  if (args[0] === "key") g.hasKey = true;
  else if (args[0] === "coin") g.coins += 1;
  else if (args[0] === "door") openDoor(world, g);
  else throw new Error("write give:<key|coin|door>");
}

/** `face:<down|up|left|right>`: turn the hero. */
function face(world: World, args: readonly string[]): void {
  const f = { down: Facing.Down, up: Facing.Up, left: Facing.Left, right: Facing.Right }[args[0] ?? ""];
  if (f === undefined) throw new Error("write face:<down|up|left|right>");
  must(world.resource(Game)).facing = f;
}

const round = (n: number) => Math.round(n * 100) / 100;

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Control, Slimes, Pickups, Door, Voices, Hud],
  render: { antialias: "off" },
  start,
  commands: { place, slime, give, face, bite: biteHero },
  commandHelp: {
    place: { usage: "<x>:<y>", doc: "Put the hero's feet at a world point, standing still." },
    slime: { usage: "<x>:<y>", doc: "Move the slime there." },
    give: { usage: "<key|coin|door>", doc: "Give the key or a coin, or open the doorway." },
    face: { usage: "<down|up|left|right>", doc: "Turn the hero." },
    bite: { usage: "<slime>", doc: "The slime's attack (og.ai queues it with the slime's handle): hurts a hero within 0.85 m." },
  },
  facts: (world) => {
    const g = world.resource(Game);
    const hero = world.find("hero");
    const map = world.find("tomb");
    if (!g.ok || hero === null || map === null) return null;
    const v = g.value;
    const h = world.get(hero, Hero);
    const slimeE = world.find("slime");
    const s = slimeE === null ? null : world.get(slimeE, og.Transform);
    return {
      room: v.room,
      hp: v.hp,
      coins: v.coins,
      hasKey: v.hasKey,
      enemyHp: v.enemyHp,
      doorOpen: v.doorOpen,
      doorWall: og.tilemap.getTile(world, map, "walls", v.doorCellX, v.doorCellY),
      hero: { x: round(v.heroX), y: round(v.heroY) },
      slime: s !== null && s.ok ? { x: round(s.value.position.x), y: round(s.value.position.y) } : null,
      facing: ["down", "up", "left", "right"][v.facing],
      steps: v.steps,
      swings: v.swings,
      hits: v.enemyHits,
      hurts: v.hurts,
      dashes: h.ok ? h.value.dashes : 0,
    };
  },
});
