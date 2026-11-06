// The 2D starter's entry: its default export is og.game(...). `og run .`
// runs it headless with the probe; `og dev .` reloads it on every save;
// `og build .` bundles it for the play page.
//
// A courier runs and jumps the rooftops of one level: a hand-written
// tilemap (assets/maps/level.ogt) whose roofs collide and whose ledges are
// one-way on their own collider layer, an og.PlatformerController moved by
// the arrows, WASD, Space or a pad, the courier's flipbook clips chosen by
// what the controller did, coins to take, a pit that costs a life, a goal
// at the far end, a camera that follows, og.ui's HUD components and the
// game's pause screen (assets/ui/pause.ogt).
//
// Every courier is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the run plays alone, two on one device (a second
// courier, the camera keeping both) or two over a room in lockstep, as
// src/creation.ts says.
// Everything the starter shows is in og-game §8 (sprites, flipbooks,
// tilemaps), §13 (the controller) and og-feel §12 (its tunables).
//
// Beside the spawn a lamp keeper stands on a ramp: a 2D skeletal rig
// (assets/rigs/keeper.ogt, og.Skeleton2D) whose cape is a weighted mesh the
// idle clip deforms, whose feet two-bone IK planted on the slope
// (og.IkTarget2D, the Plant system), and whose lantern lights the sprite
// layers through og.Lighting2D (og-game §8; og-anim's 2D skeletons). Sent
// along the ramp (`keeper:<x>`), it turns to face the way (a paper turn
// over a few ticks), plays its walk clip there, each foot planted while it
// carries the keeper and lifted while it swings, lets both feet land when
// it arrives, turns back and breathes again (the Walk and Plant systems).
// No foot ever moves faster than the gait's own swing.
//
// Made with the top-down-2d view (src/creation.ts), the same game is
// played from above in a courtyard instead: src/courtyard.ts, over the
// state, HUD, pause, voices and framing both share (src/run.ts).
//
// The frame's look is data: GameDesc.look names a file under assets/looks/
// that extends a preset of the style catalogue, applied before start runs.

import { og, defineComponent, defineResource, defineSystem, bool, f32, math, u32, SHAPE_BOX } from "@origingame/engine";
import type { FlipbookInfo, GameDesc, Rig2DInfo, TileObject, World } from "@origingame/engine";
import { courtyard } from "./courtyard.ts";
import { Coin, Frame, Game, Hud, LEVEL, LIVES, must, Sound, TOP_DOWN, VIEW_H, viewport, VIEWPORT_HELP, Voice, Voices } from "./run.ts";
import { held, pressedNow, SEATS, Seats, setBot, setupSeats } from "./seats.ts";

/** The courier's clips, in the sheet's file order (og-game §8): `start` checks them. */
const CLIP = { idle: 0, run: 1, jump: 2, fall: 3 } as const;
/** The ledges' collider layer (the map's `colliderLayers 2`), one-way for the courier. */
const LEDGES = 2;
/** The couriers' collider layer. */
const COURIERS = 8;
/** The courier's box (og.Collider): its feet are its position. */
const BOX = { w: 0.9, h: 1.9 } as const;
/** Below this the courier fell into the pit. */
const FALL_Y = -3;
/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not: og check's lockstep lint). */
const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);
/** The keeper's ramp: a plank from (x0, y0) on the roof rising at `slope` (tan 12°) to x1. */
const RAMP = { x0: 4.5, y0: 2, x1: 7.6, slope: 0.21256, sinHalf: 0.104528, cosHalf: 0.994522 } as const;
/** The ramp's top at x. */
const rampY = (x: number): number => RAMP.y0 + RAMP.slope * (x - RAMP.x0);
/** The keeper's ankles and its root above the ramp (its legs reach 1.1 from a hip 1.1 up: a slight bend), the lantern from its feet, in world units. */
const ANKLE = 0.12;
const LIFT = 0.08;
const LANTERN = { x: 0.25, y: 1.0 } as const;
/** The keeper's gait: the walk clip's 0.8 s cycle carries it a stride of 0.4 m (a step per foot), each foot lifted at most `lift` mid-swing. */
const WALK = { speed: 0.5, stride: 0.4, lift: 0.08 } as const;
/**
 * The fastest a foot moves (m/s): the gait's swing at its start, a stride
 * forward in half a cycle (twice the walking speed) while it rises up the
 * slope and off it (the lift's parabola, 4 lift over the swing's length,
 * at its steepest). A foot sent elsewhere (setting off, stopping, a turn)
 * steps there no faster.
 */
const FOOT_SPEED = hypot(2 * WALK.speed, 2 * WALK.speed * RAMP.slope + (8 * WALK.lift * WALK.speed) / WALK.stride);
/** Each foot's place standing: this far ahead of the keeper, or behind it. */
const STANCE = 0.15;
/**
 * A turn's ticks: the rig narrows to its edge and opens the other way
 * (og.Transform's scale x the cosine of a steady half turn, never 0) as
 * fast as its feet can follow it. A foot STANCE out moves `2 STANCE
 * sin(step / 2)` along the ramp in a tick of a half turn of `step`, which
 * stays within FOOT_SPEED's tick: each foot keeps up with its narrowing
 * leg, so the legs' IK reaches the feet on every tick of the turn (a
 * 12-tick turn left a foot behind its leg near the edge, up to 0.008 m
 * out of reach). 21 ticks at 60 Hz.
 */
const TURN_TICKS = Math.ceil(0.5 + Math.PI / (2 * math.asin(FOOT_SPEED / 60 / (2 * STANCE * hypot(1, RAMP.slope)))));

/** A courier: its spawn and its seat's buttons last tick (for presses). */
export const Hero = defineComponent("Hero", { spawnX: f32(0, { unit: "m" }), spawnY: f32(0, { unit: "m" }), before: u32() });

/** What the camera follows: the couriers' middle (one courier: the courier). */
export const View = defineComponent("View", {});

/**
 * One of the keeper's feet: its IK target stands on the ramp `dx` ahead of
 * the keeper (behind when negative), `lag` of a gait cycle after the front
 * foot; `x`, `y` is where the foot is.
 */
export const Plant = defineComponent("Plant", {
  dx: f32(0, { unit: "m" }),
  lag: f32(0, { min: 0, max: 1 }),
  x: f32(0, { unit: "m" }),
  y: f32(0, { unit: "m" }),
});

/** Where the keeper walks (`keeper:<x>`), how far it has walked since it set off (its gait's phase), its turn and stop, and its clips. */
export const KeeperWalk = defineResource("KeeperWalk", {
  goal: f32(6, { unit: "m" }),
  walked: f32(0, { unit: "m" }),
  walking: bool(),
  /** -1 facing left (towards the spawn), 1 right. */
  facing: f32(-1),
  /** Ticks of a turn done (0: not turning). */
  turning: u32(),
  /** og.Transform's scale x: 1, or through a turn from 1 towards -1. */
  squash: f32(1),
  /** Arrived, the feet stepping to stand (Plant sets `landed` when both are down). */
  settling: bool(),
  landed: bool(true),
  idle: u32(),
  walk: u32(),
});

/**
 * Each seat's og.PlayerInput to its courier's controller, on the press's
 * tick (og-feel §1): the run, the held jump (released early, the jump is
 * lower), Down to drop through a ledge. The jump's sound starts on the
 * press while the controller can jump (og-feel §12). A seat's PAUSE opens
 * or closes the pause menu for everyone. A fall into the pit costs a life
 * and puts the courier back at its spawn; the last life starts the level
 * again. The View follows the couriers' middle.
 */
export const Control = defineSystem(
  "Control",
  { reads: [og.PlayerInput], writes: [Game, Hero, og.PlatformerController, og.Transform] },
  (q, _time, world) => {
    const g = must(world.resource(Game));
    let sx = 0, sy = 0, n = 0;
    // Pause first: two seats pressing it on one tick toggle it once.
    let toggle = false;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const hero = a.column(Hero);
      for (let i = 0; i < a.count; i++) toggle ||= pressedNow(input.buttons[i]!, hero.before[i]!, "pause");
    }
    if (toggle) g.paused = !g.paused;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const hero = a.column(Hero);
      const pc = a.column(og.PlatformerController);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        const buttons = input.buttons[i]!;
        const playing = !g.won && !g.paused;
        pc.moveX[i] = playing ? input.x[i]! : 0;
        pc.jump[i] = playing && held(buttons, "jump") ? 1 : 0;
        pc.down[i] = playing && input.y[i]! < -0.5 ? 1 : 0;
        if (playing && pressedNow(buttons, hero.before[i]!, "jump") && (pc.grounded[i] === 1 || pc.coyote[i]! > 0)) g.sfx |= Sound.Jump;
        hero.before[i] = buttons;
        if (p[i * 3 + 1]! < FALL_Y) {
          // The last life: Pickups puts every coin back (a second courier falling on that tick costs nothing more).
          if (!g.restart) {
            g.lives -= 1;
            if (g.lives === 0) g.restart = true;
          }
          p[i * 3] = hero.spawnX[i]!;
          p[i * 3 + 1] = hero.spawnY[i]!;
          pc.velocity[i * 3] = pc.velocity[i * 3 + 1] = 0;
        }
        if (input.player[i] === 0) {
          g.heroX = p[i * 3]!;
          g.heroY = p[i * 3 + 1]!;
        } else {
          g.hero2X = p[i * 3]!;
          g.hero2Y = p[i * 3 + 1]!;
        }
        sx += p[i * 3]!;
        sy += p[i * 3 + 1]!;
        n += 1;
      }
    }
    const view = world.find("view");
    if (view !== null && n > 0) must(world.set(view, og.Transform, { position: { x: sx / n, y: sy / n, z: 0 } }));
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
      const near = (hx: number, hy: number) => {
        const dy = p[i * 3 + 1]! - hy;
        return Math.abs(p[i * 3]! - hx) < BOX.w / 2 + 0.35 && dy > -0.35 && dy < BOX.h + 0.35;
      };
      if (near(g.heroX, g.heroY) || (SEATS > 1 && near(g.hero2X, g.hero2Y))) {
        coin.taken[i] = 1;
        sprite.visible[i] = 0;
        g.coins += 1;
        g.sfx |= Sound.Coin;
      }
    }
  }
  if (!g.won && (g.heroX >= g.goalX || (SEATS > 1 && g.hero2X >= g.goalX))) {
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

/**
 * Walks the keeper towards KeeperWalk.goal along the ramp at the gait's
 * speed (by scaled time, as the clip plays: paused, both hold), facing the
 * way it goes, with the walk clip. Facing the other way first, it turns
 * over TURN_TICKS (og.Transform's scale x eased from 1 to -1, never 0, then
 * flipX), its feet stepping across with it. On arrival it holds the walk
 * pose until Plant has both feet down, turns to face the spawn, and once
 * they are down again breathes with the idle clip (which starts over,
 * breathing in).
 */
export const Walk = defineSystem("Walk", { writes: [KeeperWalk, og.Skeleton2D, og.Transform], after: ["Control"] }, (_q, time, world) => {
  const keeper = world.find("keeper");
  if (keeper === null) return;
  const w = must(world.resource(KeeperWalk));
  const x = must(world.get(keeper, og.Transform)).position.x;
  const s = must(world.get(keeper, og.Skeleton2D));
  const place = (nx: number) => must(world.set(keeper, og.Transform, { position: { x: nx, y: rampY(nx) + LIFT, z: 0 }, scale: { x: w.squash, y: 1, z: 1 } }));
  if (w.turning > 0 || (time.scaledDt > 0 && w.turning === 0 && wantsTurn(w, x))) {
    // A turn: the rig narrows to its edge and opens facing the other way.
    if (time.scaledDt === 0) return;
    w.turning += 1;
    if (w.turning < TURN_TICKS) {
      w.squash = math.cos((Math.PI * (w.turning - 0.5)) / (TURN_TICKS - 0.5));
    } else {
      w.turning = 0;
      w.squash = 1;
      w.facing = -w.facing;
      must(world.set(keeper, og.Skeleton2D, { flipX: w.facing < 0 }));
    }
    place(x);
    return;
  }
  const left = w.goal - x;
  if (left !== 0) {
    const step = WALK.speed * time.scaledDt;
    const arrived = Math.abs(left) <= step;
    const nx = arrived ? w.goal : x + Math.sign(left) * step;
    w.walked = (w.walking ? w.walked : 0) + Math.abs(nx - x);
    w.walking = !arrived;
    w.settling = arrived;
    w.landed = false;
    place(nx);
    // Walking plays the walk clip from its first step; arrived, it holds its pose while the feet land.
    if (s.clip !== w.walk) must(world.set(keeper, og.Skeleton2D, { clip: w.walk, speed: 1 }));
    else if (arrived) must(world.set(keeper, og.Skeleton2D, { speed: 0 }));
    return;
  }
  if (w.settling && w.landed) w.settling = false;
  // Standing, both feet down and facing the spawn: breathe.
  if (!w.settling && w.landed && w.facing < 0 && s.clip !== w.idle) must(world.set(keeper, og.Skeleton2D, { clip: w.idle, speed: 1 }));
});

/** Whether the keeper must turn before it goes on: towards its goal, or back to the spawn once it stands with both feet down. */
function wantsTurn(w: { goal: number; facing: number; settling: boolean; walking: boolean }, x: number): boolean {
  const left = w.goal - x;
  if (left !== 0) return Math.sign(left) !== w.facing && !w.walking;
  return !w.settling && w.facing > 0;
}

/**
 * Puts the keeper's feet on the ramp (og.IkTarget2D: the legs' two-bone IK
 * reaches each target, the knees bend as the hip bobs). Standing, each foot
 * is `dx` from the keeper; walking, the front foot's gait phase is the
 * distance walked over the stride and the back foot's half a cycle later:
 * for half a cycle a foot stays where it is on the plank while the keeper
 * passes over it, for the other half it swings a stride forward, lifted on
 * a parabola. A foot steps to where it should be at most FOOT_SPEED fast,
 * lifted while it is still on its way, so turning, setting off and
 * stopping are steps, not jumps (each leg hangs from its own hip: the
 * front foot crosses to the other side in a turn). Keeps the lantern's light at its hand, and
 * counts the clips' breaths and footfalls.
 */
export const Plant2D = defineSystem("Plant", { reads: [og.Skeleton2DEvent], writes: [Game, KeeperWalk, Plant, og.IkTarget2D, og.Transform], after: ["Walk"] }, (q, time, world) => {
  const keeper = world.find("keeper");
  if (keeper === null) return;
  const at = must(world.get(keeper, og.Transform)).position;
  const w = must(world.resource(KeeperWalk));
  const reach = FOOT_SPEED * time.scaledDt;
  let down = true;
  for (const a of q.archetypes()) {
    const plant = a.column(Plant);
    const target = a.column(og.IkTarget2D).target;
    for (let i = 0; i < a.count; i++) {
      let ahead = plant.dx[i]!;
      let lift = 0;
      if (w.walking) {
        const phase = w.walked / WALK.stride + plant.lag[i]!;
        const p = phase - Math.floor(phase);
        if (p < 0.5) {
          // Planted: it slides back under the keeper as fast as the keeper walks.
          ahead += WALK.stride * (0.25 - p);
        } else {
          const s = (p - 0.5) * 2;
          ahead += WALK.stride * (s - 0.5) * 0.5;
          lift = WALK.lift * 4 * s * (1 - s);
        }
      }
      // Turning, each foot follows its hip as the rig narrows and opens the other way.
      const wx = at.x + w.facing * w.squash * ahead;
      const [x, y] = toward(plant.x[i]!, plant.y[i]!, wx, rampY(wx) + ANKLE + lift, reach);
      // Still on its way: lifted with how far it has left (a step's height at half a stride), as fast as it may rise.
      const rest = Math.abs(wx - x);
      const [fx, fy] = rest > 1e-6 ? toward(plant.x[i]!, plant.y[i]!, x, Math.max(y, rampY(x) + ANKLE + WALK.lift * Math.min(1, rest / (WALK.stride / 2))), reach) : [x, y];
      plant.x[i] = fx;
      plant.y[i] = fy;
      target[i * 2] = fx;
      target[i * 2 + 1] = fy;
      if (hypot(wx - fx, rampY(wx) + ANKLE + lift - fy) > 1e-5 || lift > 0) down = false;
    }
  }
  w.landed = down;
  const lantern = world.find("lantern");
  if (lantern !== null) must(world.set(lantern, og.Transform, { position: { x: at.x - w.facing * w.squash * LANTERN.x, y: at.y + LANTERN.y, z: 0 } }));
  const g = must(world.resource(Game));
  for (const e of world.events(og.Skeleton2DEvent)) {
    if (e.name === "inhale") g.inhales += 1;
    else if (e.name === "step") g.steps += 1;
  }
});

/** (x, y) moved towards (tx, ty) by at most `reach`. */
function toward(x: number, y: number, tx: number, ty: number, reach: number): [number, number] {
  const d = hypot(tx - x, ty - y);
  if (d <= reach) return [tx, ty];
  return [x + ((tx - x) * reach) / d, y + ((ty - y) * reach) / d];
}

/**
 * The lamp keeper: a plank ramp, the rig on it facing the spawn, a foot
 * target per leg, and its lantern's light; og.Lighting2D lights layers 1
 * and 2 (the keeper, the courier, the coins), the city stays as drawn.
 */
function spawnKeeper(world: World, rig: Rig2DInfo): void {
  const idle = og.anim.clip(rig, "idle");
  const walk = og.anim.clip(rig, "walk");
  if (idle < 0 || walk < 0) throw new Error("rigs/keeper.ogt needs clips idle and walk");
  const mid = (RAMP.x0 + RAMP.x1) / 2;
  const plank = must(world.spawn({ name: "ramp", position: { x: mid, y: rampY(mid) - 0.2, z: 0 }, quaternion: { x: 0, y: 0, z: RAMP.sinHalf, w: RAMP.cosHalf } }));
  // The plank is a region of the keeper's texture (pixels 24..64 x 32..40 of 64 x 64).
  must(world.insert(plank, og.Sprite, { texture: rig.texture, width: RAMP.x1 - RAMP.x0, height: 0.4, uvMin: { x: 0.375, y: 0.5 }, uvMax: { x: 1, y: 0.625 }, layer: 1 }));
  const x = 6;
  const keeper = must(world.spawn({ name: "keeper", position: { x, y: rampY(x) + LIFT, z: 0 } }));
  must(world.insert(keeper, og.Skeleton2D, { rig: rig.rig, clip: idle, flipX: true, layer: 1 }));
  const w = must(world.resource(KeeperWalk));
  Object.assign(w, { goal: x, walked: 0, walking: false, facing: -1, turning: 0, squash: 1, settling: false, landed: true, idle, walk });
  // Facing left, the front foot is the left one; the back foot steps half a cycle later.
  for (const [name, ahead, lag] of [["leg-f", STANCE, 0], ["leg-b", -STANCE, 0.5]] as const) {
    const constraint = rig.constraints.indexOf(name);
    if (constraint < 0) throw new Error(`rigs/keeper.ogt has no ik ${name}`);
    const fx = x - ahead;
    const foot = must(world.spawn({ name: `keeper-${name}` }));
    must(world.insert(foot, og.IkTarget2D, { skeleton: keeper, constraint, target: { x: fx, y: rampY(fx) + ANKLE }, tolerance: 0.02 }));
    must(world.insert(foot, Plant, { dx: ahead, lag, x: fx, y: rampY(fx) + ANKLE }));
  }
  const light = must(world.spawn({ name: "lantern", position: { x: x + LANTERN.x, y: rampY(x) + LIFT + LANTERN.y, z: 0 } }));
  must(world.insert(light, og.Light2D, { color: { r: 1, g: 0.72, b: 0.35 }, intensity: 1.6, radius: 4, castShadows: false }));
  const lighting = must(world.resource(og.Lighting2D));
  lighting.enabled = true;
  lighting.ambient = { r: 0.62, g: 0.64, b: 0.78 };
  lighting.minLayer = 1;
  lighting.maxLayer = 2;
}

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

/** The rooftops, side-on (side-2d). */
const rooftops: GameDesc = {
  tick: 60,
  seed: 1,
  resources: [Game, KeeperWalk],
  systems: [Seats, Control, Frame, Pickups, Animate, Walk, Plant2D, Hud, Voices],
  start(world) {
    setupSeats(world);
    const map = must(og.load.tilemap(world, "maps/level.ogt"));
    const courier = must(og.load.flipbook(world, "sprites/courier.ogt"));
    // The second seat's courier wears the same frames in a cool palette (sprites/courier-p2.ogt).
    const courier2 = SEATS > 1 ? must(og.load.flipbook(world, "sprites/courier-p2.ogt")) : courier;
    checkClips(courier);
    const coin = must(og.load.flipbook(world, "sprites/coin.ogt"));
    spawnKeeper(world, must(og.load.rig2d(world, "rigs/keeper.ogt")));
    const height = map.height * map.tileHeight;
    LEVEL.width = map.width * map.tileWidth;
    LEVEL.height = height;

    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = VIEW_H;
    camera.position = { x: 8, y: height / 2, z: 50 };
    camera.background = { r: 0.0042, g: 0.0048, b: 0.018 };
    // A body that has rested for half a second sleeps until something touches
    // it, and costs nothing meanwhile (og.Physics.sleepTicks, 0 to keep every body awake).
    must(world.resource(og.Physics)).sleepTicks = 30;

    // The map draws from its top-left corner and gives its collision layers static colliders.
    const level = must(world.spawn({ name: "level", position: { x: 0, y: height, z: 0 } }));
    must(world.insert(level, og.Tilemap, { map: map.map, layer: -10 }));

    const g = must(world.resource(Game));
    for (const o of map.objects) {
      const p = at(o, height);
      if (o.kind === "spawn") {
        // One courier per seat: seat 1 a step behind, in the second player's palette.
        for (let seat = 0; seat < SEATS; seat++) {
          const x = p.x - seat * 1.2;
          const hero = must(world.spawn({ name: seat === 0 ? "hero" : `hero-${seat}`, position: { x, y: p.y, z: 0 } }));
          must(world.insert(hero, Hero, { spawnX: x, spawnY: p.y }));
          must(world.insert(hero, og.PlayerInput, { player: seat }));
          const sheet = seat > 0 ? courier2 : courier;
          must(world.insert(hero, og.Sprite, { texture: sheet.texture, width: 2, height: 2.5, anchor: { x: 0.5, y: 0 }, layer: 2 }));
          must(world.insert(hero, og.Flipbook, { sheet: sheet.sheet, clip: CLIP.idle }));
          // On their own layer, which no courier's mask takes: co-op couriers pass through each other.
          must(world.insert(hero, og.Collider, { shape: SHAPE_BOX, size: { x: BOX.w, y: BOX.h, z: 1 }, offset: { x: 0, y: BOX.h / 2, z: 0 }, layers: COURIERS, mask: ~COURIERS >>> 0 }));
          // og-feel §12's platformer tunables; the ledges' layer is one-way.
          must(world.insert(hero, og.PlatformerController, { maxSpeed: 8, jumpHeight: 3.6, timeToApex: 0.4, oneWayLayers: LEDGES, wallJumps: false }));
        }
        // The camera follows the courier (two couriers: Frame frames them both).
        const view = must(world.spawn({ name: "view", position: { x: p.x, y: p.y, z: 0 } }));
        must(world.insert(view, View));
        if (SEATS === 1) must(world.insert(view, og.Follow2D, { offset: { x: 0, y: 1.5 }, deadZone: { x: 1, y: 0.8 }, smoothTime: 0.12, lookAhead: 0.2, lookAheadMax: { x: 2, y: 1 } }));
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

    // Every string the HUD and the pause show (og-i18n §1), its font and the pause screen.
    must(og.i18n.load(world, "strings/en.ogt"));
    must(og.load.document(world, "ui/pause.ogt"));
    const font = og.load.font(world, "fonts/ui.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: {
    viewport,
    /** `bot:<n>`: this device's seats play a scripted run (0 hands them back to the player). */
    bot: (_world, args) => setBot(Number(args[0])),
    /** `warp:<x>:<y>`: put the courier's feet there. */
    warp: (world, args) => {
      const hero = world.find("hero");
      const x = Number(args[0]);
      const y = Number(args[1]);
      if (hero === null || !Number.isFinite(x) || !Number.isFinite(y)) throw new Error("write warp:<x>:<y>");
      must(world.set(hero, og.Transform, { position: { x, y, z: 0 } }));
    },
    /** `keeper:<x>`: send the keeper walking along the ramp to x (the Walk system moves it; its feet step on the plank). */
    keeper: (world, args) => {
      const x = Math.max(RAMP.x0 + 0.4, Math.min(RAMP.x1 - 0.4, Number(args[0])));
      if (world.find("keeper") === null || !Number.isFinite(x)) throw new Error("write keeper:<x>");
      must(world.resource(KeeperWalk)).goal = x;
    },
  },
  commandHelp: {
    viewport: VIEWPORT_HELP,
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
    warp: { usage: "<x>:<y>", doc: "Put the courier's feet at a world point." },
    keeper: { usage: "<x>", doc: "Send the lamp keeper walking along its ramp to x." },
  },
  facts: (world) => {
    const g = world.resource(Game);
    if (!g.ok) return {};
    return { coins: g.value.coins, total: g.value.total, lives: g.value.lives, won: g.value.won, inhales: g.value.inhales, steps: g.value.steps, x: g.value.heroX, y: g.value.heroY, seats: SEATS, ...(SEATS > 1 ? { x2: g.value.hero2X, y2: g.value.hero2Y } : {}) };
  },
};

export default og.game({
  // The art direction as data (assets/looks/origin-game-2d.ogt; og-taste §12).
  look: "looks/origin-game-2d.ogt",
  // src/creation.ts's view: the courtyard from above (src/courtyard.ts), or the rooftops side-on.
  ...(TOP_DOWN ? courtyard : rooftops),
});
