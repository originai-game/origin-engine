// The 2D starter's top-down-2d view (src/creation.ts's view; src/main.ts
// plays this game in place of the rooftops): the courier's courtyard seen
// from above, in the rooftops' pixel style (tools/make-courtyard.py drew
// it). The courier walks in any direction as og.CharacterController in its
// top-down mode (gravityScale 0), facing down, up or to the side; Dash
// sends it a burst along the way it walks, with ghosts behind it
// (og.Afterimage), a sound and a cooldown that flashes the courier when it
// is ready again. A canal crosses the courtyard: walking into it costs a
// life and puts the courier back on the bank it left, a dash carries it
// over. Coins lie about the yard and the north bank; the gate beside the
// lamp keeper clears the courtyard. The HUD, the pause screen, the voices
// and the two-seat framing are the rooftops' (src/run.ts).
//
// Every courier is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the courtyard plays alone, two on one device (the
// second courier in its own palette, the camera framing both) or two over
// a room in lockstep, with the touch controls' stick and Dash on a phone.

import { og, defineComponent, defineResource, defineSystem, f32, u32, SHAPE_BOX } from "@origingame/engine";
import type { FlipbookInfo, GameDesc, Rig2DInfo, TileObject, World } from "@origingame/engine";
import { Coin, Frame, Game, Hud, LEVEL, LIVES, must, Sound, VIEW_H, viewport, VIEWPORT_HELP, Voice, Voices } from "./run.ts";
import { pressedNow, SEATS, Seats, setBot, setupSeats } from "./seats.ts";

/** The facings, in each action's clip order (down, up, side); left is the side clip flipped. */
const DOWN = 0;
const UP = 1;
const RIGHT = 2;
const LEFT = 3;
/** Each action's first clip in sprites/courier-top.ogt (the facing's offset is added): `start` checks them. */
const CLIP = { idle: 0, walk: 3, dash: 6 } as const;
const FACE_NAMES = ["down", "up", "side"] as const;
/** The couriers' collider layer: co-op couriers pass through each other. */
const COURIERS = 8;
/** The courier's feet (og.Collider): a box at its position, narrower than the sprite, as a top-down character's are. */
const FEET = { w: 0.7, h: 0.4 } as const;
/** How far from the canal the courier must stand for the ground to count as the bank it comes back to. */
const BANK = 0.4;
/** Ticks the courier sits out after the canal (no input), and flickers. */
const STUN_TICKS = 14;
const HURT_TICKS = 50;
/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not: og check's lockstep lint). */
const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

/** A courier from above: its spawn, its seat's buttons last tick, its facing, its dash, the bank it last stood on and its stumble. */
export const Walker = defineComponent("Walker", {
  spawnX: f32(0, { unit: "m" }),
  spawnY: f32(0, { unit: "m" }),
  before: u32(),
  facing: u32(DOWN, { doc: "0 down, 1 up, 2 right, 3 left." }),
  dash: u32(0, { unit: "ticks", doc: "Ticks left in the dash." }),
  cooldown: u32(0, { unit: "ticks", doc: "Ticks until the next dash." }),
  dashX: f32(0),
  dashY: f32(0),
  dashes: u32(0, { doc: "Dashes so far." }),
  safeX: f32(0, { unit: "m", doc: "The last firm ground away from the canal." }),
  safeY: f32(0, { unit: "m" }),
  hurt: u32(0, { unit: "ticks", doc: "Ticks left of the stumble after the canal." }),
  falls: u32(0, { doc: "Times in the canal." }),
});

/** The courtyard's places (from the map's objects) and its tunables (og-feel §12). */
export const Yard = defineResource("Yard", {
  speed: f32(5, { min: 2, max: 10, step: 0.25, unit: "m/s", tunable: true, label: "Walk speed" }),
  dashSpeed: f32(14, { min: 6, max: 24, step: 0.5, unit: "m/s", tunable: true, label: "Dash speed" }),
  dashTicks: u32(9, { min: 2, max: 30, step: 1, unit: "ticks", tunable: true, label: "Dash length", doc: "With the speed, how far a dash goes: 9 ticks at 14 m/s is 2.1 m, twice the canal." }),
  dashCooldown: u32(32, { min: 0, max: 120, step: 1, unit: "ticks", tunable: true, label: "Dash cooldown" }),
  splashShake: f32(0.35, { min: 0, max: 1, step: 0.05, tunable: true, label: "Canal shake", doc: "Camera trauma when a courier falls in." }),
  canalX0: f32(0, { unit: "m" }),
  canalY0: f32(0, { unit: "m" }),
  canalX1: f32(0, { unit: "m" }),
  canalY1: f32(0, { unit: "m" }),
  goalX0: f32(0, { unit: "m" }),
  goalY0: f32(0, { unit: "m" }),
  goalX1: f32(0, { unit: "m" }),
  goalY1: f32(0, { unit: "m" }),
});

/** Whether (x, y) is inside a rectangle of the yard. */
const inside = (x: number, y: number, x0: number, y0: number, x1: number, y1: number, pad = 0) => x > x0 - pad && x < x1 + pad && y > y0 - pad && y < y1 + pad;

/** The facing a move points to: the stronger axis, and a diagonal keeps the old facing when it is one of its two keys. */
function faceOf(x: number, y: number, old: number): number {
  const keeps = (old === LEFT && x < 0) || (old === RIGHT && x > 0) || (old === UP && y > 0) || (old === DOWN && y < 0);
  if (Math.abs(x) > 0.01 && Math.abs(y) > 0.01 && keeps) return old;
  if (Math.abs(x) >= Math.abs(y)) return x < 0 ? LEFT : RIGHT;
  return y < 0 ? DOWN : UP;
}

function facingVector(f: number): { x: number; y: number } {
  return f === UP ? { x: 0, y: 1 } : f === DOWN ? { x: 0, y: -1 } : f === LEFT ? { x: -1, y: 0 } : { x: 1, y: 0 };
}

/**
 * Each seat's og.PlayerInput to its courier on the press's tick (og-feel
 * §1): the move walks it in any direction (a diagonal no faster than
 * straight), Dash sends it along the move (or the way it faces) for
 * Yard.dashTicks at Yard.dashSpeed, its ghosts on, and starts the
 * cooldown, which flashes the courier when it runs out. A courier whose
 * feet come down in the canal outside a dash loses a life, splashes (the
 * sound, a shake, a short hit stop, a flash) and stands again on the bank
 * it left, flickering; the last life starts the courtyard again from the
 * spawn. A seat's PAUSE opens or closes the pause menu for everyone. The
 * clip is the action's in the facing's direction, and the View follows
 * the couriers' middle.
 */
export const Control = defineSystem(
  "Control",
  { reads: [og.PlayerInput, Yard], writes: [Game, Walker, og.CharacterController, og.Transform, og.Flipbook, og.Sprite, og.Afterimage] },
  (q, time, world) => {
    const g = must(world.resource(Game));
    const yard = must(world.resource(Yard));
    let sx = 0, sy = 0, n = 0;
    // Pause first: two seats pressing it on one tick toggle it once.
    let toggle = false;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const walker = a.column(Walker);
      for (let i = 0; i < a.count; i++) toggle ||= pressedNow(input.buttons[i]!, walker.before[i]!, "pause");
    }
    if (toggle) g.paused = !g.paused;
    let splash = false;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const walker = a.column(Walker);
      const move = a.column(og.CharacterController).move;
      const p = a.column(og.Transform).position;
      const fb = a.column(og.Flipbook);
      const sprite = a.column(og.Sprite);
      const ghost = a.column(og.Afterimage);
      for (let i = 0; i < a.count; i++) {
        const buttons = input.buttons[i]!;
        const dashPressed = pressedNow(buttons, walker.before[i]!, "dash");
        walker.before[i] = buttons;
        const x = p[i * 3]!;
        const y = p[i * 3 + 1]!;
        if (input.player[i] === 0) {
          g.heroX = x;
          g.heroY = y;
        } else {
          g.hero2X = x;
          g.hero2Y = y;
        }
        sx += x;
        sy += y;
        n += 1;
        // A hit stop holds the body (physics is frozen); the timers wait with it.
        if (time.hitStop > 0) continue;
        const playing = !g.won && !g.paused;
        if (!playing) {
          move[i * 3] = move[i * 3 + 1] = 0;
          ghost.active[i] = 0;
          continue;
        }

        if (walker.hurt[i]! > 0) walker.hurt[i] = walker.hurt[i]! - 1;
        const stunned = walker.hurt[i]! > HURT_TICKS - STUN_TICKS;
        sprite.opacity[i] = walker.hurt[i]! > 0 && Math.floor(walker.hurt[i]! / 4) % 2 === 1 ? 0.35 : 1;
        if (walker.cooldown[i]! > 0) {
          walker.cooldown[i] = walker.cooldown[i]! - 1;
          // Ready again: a quick flash says so.
          if (walker.cooldown[i] === 0) must(og.tween.flash(world, { entity: a.entity(i), duration: 0.1, amount: 0.6 }));
        }

        // The move: WASD, the arrows, a stick, a d-pad or the touch stick (src/seats.ts), a diagonal at most 1 long.
        const len = hypot(input.x[i]!, input.y[i]!);
        const mx = stunned ? 0 : len > 1 ? input.x[i]! / len : input.x[i]!;
        const my = stunned ? 0 : len > 1 ? input.y[i]! / len : input.y[i]!;
        const moving = mx !== 0 || my !== 0;
        if (moving && walker.dash[i] === 0) walker.facing[i] = faceOf(mx, my, walker.facing[i]!);
        if (dashPressed && !stunned && walker.dash[i] === 0 && walker.cooldown[i] === 0) {
          const d = moving ? { x: mx / hypot(mx, my), y: my / hypot(mx, my) } : facingVector(walker.facing[i]!);
          walker.dash[i] = yard.dashTicks;
          walker.cooldown[i] = yard.dashTicks + yard.dashCooldown;
          walker.dashX[i] = d.x;
          walker.dashY[i] = d.y;
          walker.dashes[i] = walker.dashes[i]! + 1;
          g.sfx |= Sound.Dash;
        }
        const dashing = walker.dash[i]! > 0;
        if (dashing) {
          walker.dash[i] = walker.dash[i]! - 1;
          move[i * 3] = walker.dashX[i]! * yard.dashSpeed;
          move[i * 3 + 1] = walker.dashY[i]! * yard.dashSpeed;
        } else {
          move[i * 3] = mx * yard.speed;
          move[i * 3 + 1] = my * yard.speed;
        }
        move[i * 3 + 2] = 0;
        ghost.active[i] = dashing ? 1 : 0;

        // The canal: feet in the water outside a dash cost a life (the courier stands again on the bank, out of it).
        const wet = inside(x, y, yard.canalX0, yard.canalY0, yard.canalX1, yard.canalY1);
        if (wet && !dashing) {
          // The last life: Pickups puts every coin back, and this courier starts again at the spawn (a second courier falling on that tick costs nothing more).
          if (!g.restart) g.lives -= 1;
          walker.falls[i] = walker.falls[i]! + 1;
          walker.hurt[i] = HURT_TICKS;
          const last = g.lives === 0;
          if (last) g.restart = true;
          p[i * 3] = last ? walker.spawnX[i]! : walker.safeX[i]!;
          p[i * 3 + 1] = last ? walker.spawnY[i]! : walker.safeY[i]!;
          move[i * 3] = move[i * 3 + 1] = 0;
          walker.cooldown[i] = 0;
          g.sfx |= Sound.Splash;
          must(og.tween.flash(world, { entity: a.entity(i), duration: 0.15 }));
          splash = true;
        } else if (!dashing && !inside(x, y, yard.canalX0, yard.canalY0, yard.canalX1, yard.canalY1, BANK)) {
          walker.safeX[i] = x;
          walker.safeY[i] = y;
        }

        // The clip: a dash, a walk or a stand, in the facing's direction (left is the side flipped).
        const face = walker.facing[i]!;
        const action = walker.dash[i]! > 0 || dashing ? CLIP.dash : moving ? CLIP.walk : CLIP.idle;
        const clip = action + Math.min(face, RIGHT);
        if (fb.clip[i] !== clip) fb.clip[i] = clip;
        sprite.flipX[i] = face === LEFT ? 1 : 0;
      }
    }
    if (splash) {
      // og-feel §3 and §4: a light hit's stop and a medium shake for a fall.
      must(og.time.hitStop(world, 4));
      must(og.camera.shake(world, yard.splashShake));
    }
    const view = world.find("view");
    if (view !== null && n > 0) must(world.set(view, og.Transform, { position: { x: sx / n, y: sy / n, z: 0 } }));
  },
);

/** A coin within reach of a courier's feet is taken; a courier at the gate clears the courtyard; the last life lost puts every coin back. */
export const Pickups = defineSystem("Pickups", { reads: [og.Transform, Yard], writes: [Game, Coin, og.Sprite], after: ["Control"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  const yard = must(world.resource(Yard));
  const restart = g.restart;
  if (restart) {
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
      // The coin floats a little over the ground: reached by the feet below it or the body over them.
      const near = (hx: number, hy: number) => Math.abs(p[i * 3]! - hx) < 0.7 && p[i * 3 + 1]! - hy > -0.6 && p[i * 3 + 1]! - hy < 1.2;
      if (near(g.heroX, g.heroY) || (SEATS > 1 && near(g.hero2X, g.hero2Y))) {
        coin.taken[i] = 1;
        sprite.visible[i] = 0;
        g.coins += 1;
        g.sfx |= Sound.Coin;
      }
    }
  }
  const atGate = (x: number, y: number) => inside(x, y, yard.goalX0, yard.goalY0, yard.goalX1, yard.goalY1);
  if (!g.won && (atGate(g.heroX, g.heroY) || (SEATS > 1 && atGate(g.hero2X, g.hero2Y)))) {
    g.won = true;
    g.sfx |= Sound.Coin;
  }
});

/** The keeper's ankles and its root over the ground (its legs reach 1.1 from a hip 1.1 up: a slight bend), its feet apart, and the lantern from its feet. */
const KEEPER = { ankle: 0.12, lift: 0.08, stance: 0.15, lanternX: 0.25, lanternY: 1.0 } as const;

/**
 * The lamp keeper waits beside the gate, facing the yard: the rooftops'
 * rig (assets/rigs/keeper.ogt) breathing with its idle clip, its feet's IK
 * planted on the paving, a box of its own the couriers walk round, and its
 * lantern lighting the sprites.
 */
function spawnKeeper(world: World, rig: Rig2DInfo, x: number, y: number): void {
  const idle = og.anim.clip(rig, "idle");
  if (idle < 0) throw new Error("rigs/keeper.ogt needs a clip idle");
  const keeper = must(world.spawn({ name: "keeper", position: { x, y: y + KEEPER.lift, z: 0 } }));
  must(world.insert(keeper, og.Skeleton2D, { rig: rig.rig, clip: idle, layer: 1 }));
  must(world.insert(keeper, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 0.4, z: 1 }, offset: { x: 0, y: 0.2 - KEEPER.lift, z: 0 } }));
  // Facing right, the front foot is the right one.
  for (const [name, ahead] of [["leg-f", KEEPER.stance], ["leg-b", -KEEPER.stance]] as const) {
    const constraint = rig.constraints.indexOf(name);
    if (constraint < 0) throw new Error(`rigs/keeper.ogt has no ik ${name}`);
    const foot = must(world.spawn({ name: `keeper-${name}` }));
    must(world.insert(foot, og.IkTarget2D, { skeleton: keeper, constraint, target: { x: x + ahead, y: y + KEEPER.ankle }, tolerance: 0.02 }));
  }
  lamp(world, "lantern", x - KEEPER.lanternX, y + KEEPER.lift + KEEPER.lanternY, 1.6, 4);
}

/** A warm light over the sprite layers (og.Lighting2D's 1 and 2). */
function lamp(world: World, name: string, x: number, y: number, intensity: number, radius: number): void {
  const light = must(world.spawn({ name, position: { x, y, z: 0 } }));
  must(world.insert(light, og.Light2D, { color: { r: 1, g: 0.72, b: 0.35 }, intensity, radius, castShadows: false }));
}

function checkClips(info: FlipbookInfo): void {
  for (const [action, first] of Object.entries(CLIP)) {
    FACE_NAMES.forEach((face, k) => {
      const at = og.anim.clip(info, `${action}-${face}`);
      if (at !== first + k) throw new Error(`sprites/courier-top.ogt: clip ${action}-${face} is ${at}, the code expects ${first + k}`);
    });
  }
}

/** A map object's point in world units: the map's top-left corner is at (0, height). */
function at(o: TileObject, height: number): { x: number; y: number } {
  return { x: o.position.x, y: height + o.position.y };
}

function start(world: World): void {
  setupSeats(world);
  const map = must(og.load.tilemap(world, "maps/courtyard.ogt"));
  const courier = must(og.load.flipbook(world, "sprites/courier-top.ogt"));
  // The second seat's courier wears the same frames in the rooftops' P2 palette (sprites/courier-top-p2.ogt).
  const courier2 = SEATS > 1 ? must(og.load.flipbook(world, "sprites/courier-top-p2.ogt")) : courier;
  checkClips(courier);
  const coin = must(og.load.flipbook(world, "sprites/coin.ogt"));
  const rig = must(og.load.rig2d(world, "rigs/keeper.ogt"));
  const height = map.height * map.tileHeight;
  LEVEL.width = map.width * map.tileWidth;
  LEVEL.height = height;
  // Framing two couriers: the sprite's figure stands this tall over its feet.
  LEVEL.figure = 2.2;

  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = VIEW_H;
  camera.background = { r: 0.0042, g: 0.0048, b: 0.018 };
  const physics = must(world.resource(og.Physics));
  physics.gravity = { x: 0, y: 0, z: 0 };
  physics.sleepTicks = 30;
  const lighting = must(world.resource(og.Lighting2D));
  lighting.enabled = true;
  lighting.ambient = { r: 0.62, g: 0.64, b: 0.78 };
  lighting.minLayer = 1;
  lighting.maxLayer = 2;

  // The map draws from its top-left corner and gives its walls layer static colliders.
  const level = must(world.spawn({ name: "courtyard", position: { x: 0, y: height, z: 0 } }));
  must(world.insert(level, og.Tilemap, { map: map.map, layer: -10 }));

  const g = must(world.resource(Game));
  const yard = must(world.resource(Yard));
  for (const o of map.objects) {
    const p = at(o, height);
    if (o.kind === "spawn") {
      // One courier per seat: seat 1 a step to the left, in the second player's palette.
      for (let seat = 0; seat < SEATS; seat++) {
        const x = p.x - seat * 1.2;
        const hero = must(world.spawn({ name: seat === 0 ? "hero" : `hero-${seat}`, position: { x, y: p.y, z: 0 } }));
        must(world.insert(hero, Walker, { spawnX: x, spawnY: p.y, safeX: x, safeY: p.y, facing: UP }));
        must(world.insert(hero, og.PlayerInput, { player: seat }));
        const sheet = seat > 0 ? courier2 : courier;
        must(world.insert(hero, og.Sprite, { texture: sheet.texture, width: 2, height: 2.5, anchor: { x: 0.5, y: 0 }, layer: 2, shadow: 0.35, shadowSize: { x: 0.9, y: 0.3 }, shadowOffset: { x: 0, y: 0.05 } }));
        must(world.insert(hero, og.Flipbook, { sheet: sheet.sheet, clip: CLIP.idle + UP }));
        // Walls stop the feet; couriers on their own layer, which no courier's mask takes, pass through each other.
        must(world.insert(hero, og.Collider, { shape: SHAPE_BOX, size: { x: FEET.w, y: FEET.h, z: 1 }, offset: { x: 0, y: FEET.h / 2, z: 0 }, friction: 0, layers: COURIERS, mask: ~COURIERS >>> 0 }));
        // og.CharacterController's top-down mode: no gravity, sliding along the walls.
        must(world.insert(hero, og.CharacterController, { gravityScale: 0, snapDistance: 0, stepHeight: 0, pushBodies: false }));
        // The dash's ghosts, in the courier's scarf: warm for seat 0, ice for seat 1.
        must(world.insert(hero, og.Afterimage, { count: 4, interval: 2, opacity: 0.55, color: seat > 0 ? { r: 0.55, g: 0.85, b: 1 } : { r: 1, g: 0.6, b: 0.35 }, active: false }));
      }
      // The camera follows the courier inside the courtyard (two couriers: Frame frames them both).
      const view = must(world.spawn({ name: "view", position: { x: p.x, y: p.y, z: 0 } }));
      camera.position = { x: p.x, y: p.y + 1, z: 50 };
      if (SEATS === 1) {
        must(world.insert(view, og.Follow2D, { offset: { x: 0, y: 1 }, deadZone: { x: 0.6, y: 0.4 }, smoothTime: 0.12, lookAhead: 0.15, lookAheadMax: { x: 1.5, y: 1 } }));
        const zone = must(world.spawn({ name: "yard" }));
        must(world.insert(zone, og.CameraZone, { min: { x: 0, y: 0 }, max: { x: LEVEL.width, y: height }, scrollTime: 0 }));
      }
    } else if (o.kind === "coin") {
      // Standing on its point's ground (the map gives its middle), with a shadow under it.
      const e = must(world.spawn({ name: o.name, position: { x: p.x, y: p.y - 0.35, z: 0 } }));
      must(world.insert(e, Coin));
      must(world.insert(e, og.Sprite, { texture: coin.texture, width: 0.7, height: 0.7, anchor: { x: 0.5, y: 0 }, layer: 1, shadow: 0.3, shadowSize: { x: 0.5, y: 0.16 }, shadowOffset: { x: 0, y: 0.03 } }));
      must(world.insert(e, og.Flipbook, { sheet: coin.sheet, clip: 0 }));
      g.total += 1;
    } else if (o.kind === "hazard") {
      Object.assign(yard, { canalX0: p.x, canalY0: p.y - o.size.y, canalX1: p.x + o.size.x, canalY1: p.y });
    } else if (o.kind === "goal") {
      Object.assign(yard, { goalX0: p.x, goalY0: p.y - o.size.y, goalX1: p.x + o.size.x, goalY1: p.y });
    } else if (o.kind === "keeper") {
      spawnKeeper(world, rig, p.x, p.y);
    } else if (o.kind === "lamp") {
      lamp(world, o.name, p.x, p.y, 1.2, 3.5);
    }
  }

  for (const [path, bit] of [["audio/coin.wav", Sound.Coin], ["audio/dash.wav", Sound.Dash], ["audio/splash.wav", Sound.Splash]] as const) {
    const sound = og.load.sound(world, path);
    if (!sound.ok) continue;
    const v = must(world.spawn({ name: `voice-${bit}` }));
    must(world.insert(v, Voice, { bit }));
    must(world.insert(v, og.AudioSource, { sound: sound.value, playing: false, volume: 0.5 }));
  }

  // Every string the HUD and the pause show (og-i18n §1), its font and the pause screen.
  must(og.i18n.load(world, "strings/en.ogt"));
  must(og.load.document(world, "ui/pause.ogt"));
  const font = og.load.font(world, "fonts/ui.ttf");
  if (font.ok) og.ui.configure(world, { font: font.value });
}

/** The courtyard's game: src/main.ts gives it the look. */
export const courtyard: GameDesc = {
  tick: 60,
  seed: 1,
  resources: [Game, Yard],
  systems: [Seats, Control, ...og.systems.cameras, Frame, Pickups, Hud, Voices],
  start,
  commands: {
    viewport,
    /** `bot:<n>`: this device's seats play a scripted run (0 hands them back to the player). */
    bot: (_world, args) => setBot(Number(args[0])),
    /** `warp:<x>:<y>`: put the courier's feet there, standing still. */
    warp: (world, args) => {
      const hero = world.find("hero");
      const x = Number(args[0]);
      const y = Number(args[1]);
      if (hero === null || !Number.isFinite(x) || !Number.isFinite(y)) throw new Error("write warp:<x>:<y>");
      must(world.set(hero, og.Transform, { position: { x, y, z: 0 } }));
      must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 } }));
    },
  },
  commandHelp: {
    viewport: VIEWPORT_HELP,
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
    warp: { usage: "<x>:<y>", doc: "Put the courier's feet at a world point, standing still." },
  },
  facts: (world) => {
    const g = world.resource(Game);
    const hero = world.find("hero");
    const w = hero === null ? null : world.get(hero, Walker);
    if (!g.ok || w === null || !w.ok) return {};
    const v = g.value;
    return { coins: v.coins, total: v.total, lives: v.lives, won: v.won, x: v.heroX, y: v.heroY, seats: SEATS, dashes: w.value.dashes, falls: w.value.falls, facing: ["down", "up", "right", "left"][w.value.facing], ...(SEATS > 1 ? { x2: v.hero2X, y2: v.hero2Y } : {}) };
  },
};
