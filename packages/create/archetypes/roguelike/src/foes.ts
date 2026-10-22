// The foes' actions: the game's commands their behaviours raise (og.ai
// queues each with the foe's handle; they run before the next tick). One
// command is one foe's whole turn.
//
// - strike: the lamp-bearer is next to it: it loses the foe's strike less
//   its armour (at least 1), twice that for a charged strike; a hit holds
//   the world, flashes the lamp-bearer red, shakes and kicks the camera.
// - hunt: a step along og.nav's path to the lamp-bearer (or to where the
//   foe last saw it), never onto another foe; into the lamp-bearer it is a
//   strike.
// - roam: a step in a direction of the dice (rats roam every turn,
//   skeletons every other).
// - brace: the guardian holds its ground.
// The guardian charges every third turn: the turn before, it glows red and
// a "!" rises over it, so the lamp-bearer can step back and let the charge
// pass.

import { og } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Foe, FoeKind, FOE_SLOTS, Game, Sound } from "./components.ts";
import { H, W } from "./dive.ts";
import { cellCenter, feel, float, isWall, lazyTuning, must, must2, RED } from "./systems.ts";
import { say } from "./ui.ts";

type FoeV = Value<typeof Foe>;
type GameV = Value<typeof Game>;

interface Ctx {
  me: Entity;
  f: FoeV;
  g: GameV;
}

function context(world: World, args: readonly string[]): Ctx | null {
  const me = Number(args[0]) as Entity;
  const foe = world.get(me, Foe);
  const game = world.resource(Game);
  if (!foe.ok || !game.ok) throw new Error(`${args[0]} is not a foe`);
  const g = game.value;
  if (!foe.value.active || foe.value.dying > 0 || g.hp <= 0 || g.won) return null;
  return { me, f: foe.value as FoeV, g };
}

/**
 * Counts the foe's turn. The guardian's every third turn is charged,
 * telegraphed a turn ahead; a charge not struck with on that turn (the
 * lamp-bearer stepped away) is spent.
 */
function turnTaken(world: World, c: Ctx): void {
  const turns = c.f.turns + 1;
  const charging = c.f.kind === FoeKind.Guardian && turns % 3 === 2;
  if (charging) {
    const at = cellCenter(c.f.cx, c.f.cy);
    float(world, say(world, "float.charge"), at.x, at.y + 0.9, RED, true, 0.6);
    c.g.sfx |= Sound.Telegraph;
  }
  must(world.set(c.me, Foe, { turns, charging }));
}

function occupied(world: World, x: number, y: number, me: Entity): boolean {
  for (let i = 0; i < FOE_SLOTS; i++) {
    const e = world.find(`foe-${i}`);
    if (e === null || e === me) continue;
    const f = world.get(e, Foe);
    if (f.ok && f.value.active && f.value.cx === x && f.value.cy === y) return true;
  }
  return false;
}

function stepTo(world: World, c: Ctx, x: number, y: number): boolean {
  if (x < 0 || y < 0 || x >= W || y >= H || isWall(world, c.g, x, y) || occupied(world, x, y, c.me)) return false;
  if (x === c.g.heroX && y === c.g.heroY) return false;
  must(world.set(c.me, Foe, { cx: x, cy: y }));
  const to = cellCenter(x, y);
  must(og.tween.to(world, { entity: c.me, field: "position", to: [to.x, to.y, 0], duration: lazyTuning(world).stepTime, ease: "quadOut" }));
  return true;
}

/** `strike:<foe>`: the foe strikes the lamp-bearer beside it. */
export function strike(world: World, args: readonly string[]): void {
  const c = context(world, args);
  if (c === null) return;
  const { f, g } = c;
  if (Math.abs(f.cx - g.heroX) + Math.abs(f.cy - g.heroY) !== 1) {
    hunt(world, args);
    return;
  }
  const charged = f.charging;
  const dmg = Math.max(1, f.attack * (charged ? 2 : 1) - g.armour);
  g.hp = Math.max(0, g.hp - dmg);
  g.hurts += 1;
  g.sfx |= Sound.Hurt;
  const hero = world.find("hero");
  const at = cellCenter(g.heroX, g.heroY);
  float(world, say(world, "float.hurt", { n: dmg }), at.x, at.y + 0.7, RED, charged);
  if (hero !== null) {
    must(og.tween.flash(world, { entity: hero, color: RED, duration: 0.2 }));
    must(og.tween.squash(world, { entity: hero, impact: { x: 1.25, y: 0.75 }, duration: 0.2 }));
  }
  const from = cellCenter(f.cx, f.cy);
  // The foe lunges at the lamp-bearer and back.
  must(og.tween.sequence(world, [
    { entity: c.me, field: "position", to: [from.x + (at.x - from.x) * 0.35, from.y + (at.y - from.y) * 0.35, 0], duration: 0.05, ease: "quadOut" },
    { entity: c.me, field: "position", to: [from.x, from.y, 0], duration: 0.09, ease: "quadIn" },
  ]));
  const dead = g.hp === 0;
  must(og.time.hitStop(world, dead ? 16 : charged ? 8 : 3));
  const k = feel(world);
  must(og.camera.shake(world, (dead ? 0.7 : charged ? 0.5 : 0.25) * k));
  must(og.camera.kick(world, { x: at.x - from.x, y: at.y - from.y, z: 0 }, (charged ? 0.25 : 0.12) * k));
  if (dead) {
    g.killedBy = f.kind;
    g.overTicks = 0;
    g.sfx |= Sound.Fall;
  }
  turnTaken(world, c);
}

/** `hunt:<foe>`: a step along og.nav's path to the lamp-bearer, or to where the foe last saw it. */
export function hunt(world: World, args: readonly string[]): void {
  const c = context(world, args);
  if (c === null) return;
  const { f, g } = c;
  const eyes = world.get(c.me, og.Perception);
  const sees = eyes.ok && eyes.value.sees;
  let tx = g.heroX;
  let ty = g.heroY;
  if (!sees && eyes.ok) {
    tx = Math.floor(eyes.value.lastSeen.x);
    ty = H - 1 - Math.floor(eyes.value.lastSeen.y);
  }
  if (Math.abs(f.cx - g.heroX) + Math.abs(f.cy - g.heroY) === 1 && sees) {
    strike(world, args);
    return;
  }
  const from = cellCenter(f.cx, f.cy);
  const to = cellCenter(tx, ty);
  const path = og.nav.findPath(world, { nav: g.grid as unknown as never, from: { x: from.x, y: from.y, z: 0 }, to: { x: to.x, y: to.y, z: 0 }, smooth: false });
  let moved = false;
  if (path.ok && path.value.points.length > 1) {
    const next = path.value.points[1]!;
    const dx = Math.sign(Math.round(next.x - 0.5) - f.cx);
    const dy = Math.sign(H - 1 - Math.round(next.y - 0.5) - f.cy);
    // Along the longer leg first, then the other.
    const tries = Math.abs(next.x - from.x) >= Math.abs(next.y - from.y) ? [[dx, 0], [0, dy]] : [[0, dy], [dx, 0]];
    for (const [sx, sy] of tries) {
      if ((sx !== 0 || sy !== 0) && !moved) moved = stepTo(world, c, f.cx + sx!, f.cy + sy!);
    }
  }
  turnTaken(world, c);
}

/** `roam:<foe>`: a step in a direction of the dice (a skeleton keeps still every other turn). */
export function roam(world: World, args: readonly string[]): void {
  const c = context(world, args);
  if (c === null) return;
  const rng = must2(world.resource(og.Rng));
  if (c.f.kind === FoeKind.Rat || c.f.turns % 2 === 0) {
    const start = rng.integer(0, 3);
    const dirs = [[0, -1], [1, 0], [0, 1], [-1, 0]];
    for (let k = 0; k < 4; k++) {
      const [dx, dy] = dirs[(start + k) % 4]!;
      if (stepTo(world, c, c.f.cx + dx!, c.f.cy + dy!)) break;
    }
  }
  turnTaken(world, c);
}

/** `brace:<foe>`: the guardian holds its ground (its charge still counts). */
export function brace(world: World, args: readonly string[]): void {
  const c = context(world, args);
  if (c === null) return;
  turnTaken(world, c);
}
