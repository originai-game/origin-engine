// The 2D starter's entry: its default export is og.game(...). `og run .`
// runs it headless with the probe; `og dev .` reloads it on every save;
// `og build .` bundles it for the play page.
//
// A courier runs and jumps the rooftops of one level: a hand-written
// tilemap (assets/maps/level.ogt) whose roofs collide and whose ledges are
// one-way on their own collider layer, an og.PlatformerController moved by
// the arrows, WASD, Space or a pad, the courier's flipbook clips chosen by
// what the controller did, coins to take, a pit that costs a life, a goal
// at the far end, a camera that follows, and the UI kit's platformer HUD.
// Everything the starter shows is in og-game §8 (sprites, flipbooks,
// tilemaps), §13 (the controller) and og-feel §12 (its tunables).
//
// The frame's look is data: GameDesc.look names a file under assets/looks/
// that extends a preset of the style catalogue, applied before start runs.

import { og, defineComponent, defineResource, defineSystem, bool, f32, u32, SHAPE_BOX } from "@origingame/engine";
import type { FlipbookInfo, Result, TileObject } from "@origingame/engine";

/** The courier's clips, in the sheet's file order (og-game §8): `start` checks them. */
const CLIP = { idle: 0, run: 1, jump: 2, fall: 3 } as const;
/** The camera's height in world units: 14 cells of 16 px, the 16bit-pixel look's 224-pixel frame. */
const VIEW_H = 14;
/** The ledges' collider layer (the map's `colliderLayers 2`), one-way for the courier. */
const LEDGES = 2;
/** The courier's box (og.Collider): its feet are its position. */
const BOX = { w: 0.9, h: 1.9 } as const;
/** Below this the courier fell into the pit. */
const FALL_Y = -3;
const LIVES = 3;

/** The courier. */
export const Hero = defineComponent("Hero", { spawnX: f32(0, { unit: "m" }), spawnY: f32(0, { unit: "m" }) });

/** A coin of the map's `coin` objects; taken once. */
export const Coin = defineComponent("Coin", { taken: bool() });

/** A voice: the sound bit (Sound) it plays when Game.sfx asks for it. */
export const Voice = defineComponent("Voice", { bit: u32() });

const Sound = { Jump: 1, Coin: 2 } as const;

/** The run: what the HUD shows and the systems share. */
export const Game = defineResource("Game", {
  coins: u32(),
  total: u32(),
  lives: u32(LIVES),
  won: bool(),
  /** The last life was lost: Pickups starts the level again. */
  restart: bool(),
  goalX: f32(0, { unit: "m" }),
  heroX: f32(0, { unit: "m" }),
  heroY: f32(0, { unit: "m" }),
  /** Sounds asked for this tick (Sound bits), played by Voices. */
  sfx: u32(),
  /** The pause menu is open: the courier and time hold. */
  paused: bool(),
  /** The HUD's touch controls this tick (a phone): the run and the held jump. */
  touchX: f32(0, { min: -1, max: 1 }),
  touchJump: bool(),
});

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/**
 * Input to the controller, on the press's tick (og-feel §1): the run, the
 * held jump (released early, the jump is lower), Down to drop through a
 * ledge. The jump's sound starts on the press while the controller can
 * jump (og-feel §12). A fall into the pit costs a life and puts the
 * courier back at the spawn; the last life starts the level again.
 */
export const Control = defineSystem(
  "Control",
  { reads: [Hero, og.Input, og.Gamepad], writes: [Game, og.PlatformerController, og.Transform], after: ["Hud"] },
  (q, _time, world) => {
    const g = must(world.resource(Game));
    const input = must(world.resource(og.Input));
    const pad = must(world.resource(og.Gamepad));
    const stick = pad.axis("leftX", 0);
    const run = Math.max(-1, Math.min(1, input.axis("ArrowLeft", "ArrowRight") + input.axis("KeyA", "KeyD") + (Math.abs(stick) > 0.25 ? stick : 0) + g.touchX));
    const jump = input.down("Space") || input.down("ArrowUp") || input.down("KeyW") || pad.down("a", 0) || g.touchJump;
    const pressed = input.pressed("Space") || input.pressed("ArrowUp") || input.pressed("KeyW") || pad.pressed("a", 0);
    const down = input.down("ArrowDown") || input.down("KeyS") || pad.down("down", 0);
    for (const a of q.archetypes()) {
      const hero = a.column(Hero);
      const pc = a.column(og.PlatformerController);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        const playing = !g.won && !g.paused;
        pc.moveX[i] = playing ? run : 0;
        pc.jump[i] = playing && jump ? 1 : 0;
        pc.down[i] = playing && down ? 1 : 0;
        if (playing && pressed && (pc.grounded[i] === 1 || pc.coyote[i]! > 0)) g.sfx |= Sound.Jump;
        if (p[i * 3 + 1]! < FALL_Y) {
          g.lives -= 1;
          // The last life: Pickups puts every coin back.
          if (g.lives === 0) g.restart = true;
          p[i * 3] = hero.spawnX[i]!;
          p[i * 3 + 1] = hero.spawnY[i]!;
          pc.velocity[i * 3] = pc.velocity[i * 3 + 1] = 0;
        }
        g.heroX = p[i * 3]!;
        g.heroY = p[i * 3 + 1]!;
      }
    }
  },
);

/** A coin the courier's box touches is taken; reaching the goal clears the level; the last life lost puts every coin back. */
export const Pickups = defineSystem("Pickups", { writes: [Game, Coin, og.Sprite], reads: [og.Transform], after: ["Control"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  const restart = g.restart;
  if (restart) {
    // The last life was lost: every coin back, the lives full.
    g.restart = false;
    g.coins = 0;
    g.lives = LIVES;
  }
  for (const a of q.archetypes()) {
    const coin = a.column(Coin);
    const sprite = a.column(og.Sprite);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      if (restart) {
        coin.taken[i] = 0;
        sprite.visible[i] = 1;
      }
      if (coin.taken[i] === 1) continue;
      const dx = Math.abs(p[i * 3]! - g.heroX);
      const dy = p[i * 3 + 1]! - g.heroY;
      if (dx < BOX.w / 2 + 0.35 && dy > -0.35 && dy < BOX.h + 0.35) {
        coin.taken[i] = 1;
        sprite.visible[i] = 0;
        g.coins += 1;
        g.sfx |= Sound.Coin;
      }
    }
  }
  if (!g.won && g.heroX >= g.goalX) {
    g.won = true;
    g.sfx |= Sound.Coin;
  }
});

/** The courier's clip from what its controller did, and its facing. */
export const Animate = defineSystem("Animate", { reads: [Hero, og.PlatformerController], writes: [og.Flipbook, og.Sprite], after: ["Control"] }, (q) => {
  for (const a of q.archetypes()) {
    const pc = a.column(og.PlatformerController);
    const fb = a.column(og.Flipbook);
    const sprite = a.column(og.Sprite);
    for (let i = 0; i < a.count; i++) {
      const vx = pc.velocity[i * 3]!;
      const vy = pc.velocity[i * 3 + 1]!;
      const clip = pc.grounded[i] !== 1 ? (vy > 0 ? CLIP.jump : CLIP.fall) : Math.abs(vx) > 0.2 ? CLIP.run : CLIP.idle;
      if (fb.clip[i] !== clip) fb.clip[i] = clip;
      sprite.flipX[i] = pc.facing[i]! < 0 ? 1 : 0;
    }
  }
});

/** Plays each sound asked for once: a voice still playing is stopped this tick and played again the next. */
export const Voices = defineSystem("Voices", { reads: [Voice], writes: [Game, og.AudioSource], after: ["Pickups"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.sfx === 0) return;
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const voice = a.column(Voice);
    const source = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const bit = voice.bit[i]!;
      if ((g.sfx & bit) === 0) continue;
      if (source.playing[i]) source.playing[i] = 0;
      else {
        source.playing[i] = 1;
        left &= ~bit;
      }
    }
  }
  g.sfx = left;
});

/**
 * The UI kit's platformer HUD (og-ui §6): lives, coins, the level's name,
 * the pause button and, on a phone, the touch controls (taken by Control
 * this tick: Hud runs first). Pause (the button, Escape or the pad's Start)
 * opens the kit's pause menu and holds time until Resume.
 */
export const Hud = defineSystem("Hud", { writes: [Game] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  const hud = og.ui.platformerHud("Hud", {
    lives: g.lives,
    maxLives: LIVES,
    coins: g.coins,
    score: g.coins * 100,
    level: og.i18n.t(world, g.won ? "hud.clear" : "hud.level"),
    timeLeft: null,
    // A d-pad and Jump where the pointer is a finger (og-ui §5).
    touch: og.ui.frameViewport().coarsePointer,
  });
  g.touchX = g.paused ? 0 : hud.moveX;
  g.touchJump = !g.paused && hud.held.includes("jump");
  const menu = og.ui.pauseMenu("Pause", { open: g.paused, items: [{ id: "resume", label: og.i18n.t(world, "pause.resume") }] });
  if (g.paused && menu.selected === "resume") {
    g.paused = false;
    must(og.time.setScale(world, 1));
  } else if (!g.paused && hud.paused) {
    g.paused = true;
    must(og.time.setScale(world, 0));
  }
});

function checkClips(info: FlipbookInfo): void {
  for (const [name, index] of Object.entries(CLIP)) {
    const at = og.anim.clip(info, name);
    if (at !== index) throw new Error(`sprites/courier.ogt: clip ${name} is ${at}, the code expects ${index}`);
  }
}

/** A map object's point in world units: the map's top-left corner is at (0, height). */
function at(o: TileObject, height: number): { x: number; y: number } {
  return { x: o.position.x, y: height + o.position.y };
}

export default og.game({
  // The art direction as data (assets/looks/origin-game-2d.ogt; og-taste §12).
  look: "looks/origin-game-2d.ogt",
  tick: 60,
  seed: 1,
  resources: [Game],
  systems: [Hud, Control, Pickups, Animate, Voices],
  start(world) {
    const map = must(og.load.tilemap(world, "maps/level.ogt"));
    const courier = must(og.load.flipbook(world, "sprites/courier.ogt"));
    checkClips(courier);
    const coin = must(og.load.flipbook(world, "sprites/coin.ogt"));
    const height = map.height * map.tileHeight;

    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = VIEW_H;
    camera.position = { x: 8, y: height / 2, z: 50 };
    camera.background = { r: 0.0042, g: 0.0048, b: 0.018 };

    // The map draws from its top-left corner and gives its collision layers static colliders.
    const level = must(world.spawn({ name: "level", position: { x: 0, y: height, z: 0 } }));
    must(world.insert(level, og.Tilemap, { map: map.map, layer: -10 }));

    const g = must(world.resource(Game));
    for (const o of map.objects) {
      const p = at(o, height);
      if (o.kind === "spawn") {
        const hero = must(world.spawn({ name: "hero", position: { x: p.x, y: p.y, z: 0 } }));
        must(world.insert(hero, Hero, { spawnX: p.x, spawnY: p.y }));
        must(world.insert(hero, og.Sprite, { texture: courier.texture, width: 2, height: 2.5, anchor: { x: 0.5, y: 0 }, layer: 2 }));
        must(world.insert(hero, og.Flipbook, { sheet: courier.sheet, clip: CLIP.idle }));
        must(world.insert(hero, og.Collider, { shape: SHAPE_BOX, size: { x: BOX.w, y: BOX.h, z: 1 }, offset: { x: 0, y: BOX.h / 2, z: 0 } }));
        // og-feel §12's platformer tunables; the ledges' layer is one-way.
        must(world.insert(hero, og.PlatformerController, { maxSpeed: 8, jumpHeight: 3.6, timeToApex: 0.4, oneWayLayers: LEDGES, wallJumps: false }));
        must(world.insert(hero, og.Follow2D, { offset: { x: 0, y: 1.5 }, deadZone: { x: 1, y: 0.8 }, smoothTime: 0.12, lookAhead: 0.2, lookAheadMax: { x: 2, y: 1 } }));
      } else if (o.kind === "coin") {
        const e = must(world.spawn({ name: o.name, position: { x: p.x, y: p.y, z: 0 } }));
        must(world.insert(e, Coin));
        must(world.insert(e, og.Sprite, { texture: coin.texture, width: 0.7, height: 0.7, layer: 1 }));
        must(world.insert(e, og.Flipbook, { sheet: coin.sheet, clip: 0 }));
        g.total += 1;
      } else if (o.kind === "goal") {
        g.goalX = p.x;
      }
    }

    for (const [path, bit] of [["audio/jump.wav", Sound.Jump], ["audio/coin.wav", Sound.Coin]] as const) {
      const sound = og.load.sound(world, path);
      if (!sound.ok) continue;
      const v = must(world.spawn({ name: `voice-${bit}` }));
      must(world.insert(v, Voice, { bit }));
      must(world.insert(v, og.AudioSource, { sound: sound.value, playing: false, volume: 0.5 }));
    }

    // Every string the HUD shows (og-i18n §1), and its font.
    must(og.i18n.load(world, "strings/en.ogt"));
    const font = og.load.font(world, "fonts/ui.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: {
    /** `warp:<x>:<y>`: put the courier's feet there. */
    warp: (world, args) => {
      const hero = world.find("hero");
      const x = Number(args[0]);
      const y = Number(args[1]);
      if (hero === null || !Number.isFinite(x) || !Number.isFinite(y)) throw new Error("write warp:<x>:<y>");
      must(world.set(hero, og.Transform, { position: { x, y, z: 0 } }));
    },
  },
  commandHelp: {
    warp: { usage: "<x>:<y>", doc: "Put the courier's feet at a world point." },
  },
  facts: (world) => {
    const g = world.resource(Game);
    if (!g.ok) return {};
    return { coins: g.value.coins, total: g.value.total, lives: g.value.lives, won: g.value.won, x: g.value.heroX, y: g.value.heroY };
  },
});
