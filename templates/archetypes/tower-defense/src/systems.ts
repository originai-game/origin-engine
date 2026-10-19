// Path Guard's systems, in tick order: Control (the cursor, the tray keys,
// a tap or a key setting a plant), Plants (reloads, sight, the shooting
// clip), Visitors (the waves, the walk, hits landing, pops and leaks) and
// Voices (each requested sound once). The UI is src/ui.ts; `fire`, the
// plants' behaviour command, is here.
//
// A plant's mind is data: assets/ai/plant.ogt, run by og.ai before these
// systems, checks that the plant is set and reloaded and sees a visitor
// (og.Perception picks the nearest within range), then raises the `fire`
// command, which sends a shot along a tween. The tween's tag names the
// visitor's pool slot and the plant's kind, so Visitors applies the hit on
// the tick it lands. The visitors walk og.nav's flow field to the doorstep;
// every plant set re-costs the field's grid, and a plant that would close
// the path (for the gate or any visitor on the field) is refused.

import { og, defineSystem, NAV_ARRIVED } from "@origingame/engine";
import type { Entity, Value, View, World } from "@origingame/engine";
import { Bed, Cursor, Game, Phase, Plant, PLANTS, Refusal, Sfx, Shot, Sound, Tuning, Visitor } from "./components.ts";
import { clipOf } from "./art.ts";
import { say } from "./ui.ts";

/** The field: 9 × 13 cells of one world unit; the map's top-left corner is at (0, 13). */
export const COLS = 9;
export const ROWS = 13;
/** The gate's cell (the visitors' spawn) and the doorstep's (their goal), map columns and rows. */
export const GATE = { x: 4, y: 0 } as const;
export const DOOR = { x: 4, y: 12 } as const;
/** Pool sizes. */
export const VISITORS = 24;
export const SHOTS = 24;
/** A visitor's pop lasts this long before its row is parked. */
const POP_TICKS = 36;
/** Where parked rows wait, far outside every plant's sight. */
const PARK = { x: -40, y: -40, z: 0 } as const;

/** A visitor's base numbers by Kind.*: health, speed (m/s), bounty (coins), light it takes. */
export const KINDS = [
  { name: "beetle", hp: 34, speed: 1.3, bounty: 6, harm: 1 },
  { name: "moth", hp: 22, speed: 2.1, bounty: 5, harm: 1 },
  { name: "snail", hp: 140, speed: 0.75, bounty: 20, harm: 3 },
] as const;

/** The waves, one letter per visitor in the order they come: b beetle, m moth, s snail. */
export const WAVES = ["bbbbbb", "bbmbbmbbmb", "mmbmmbmmbmmb", "bbsbbmbbmbbsbb", "sbbmmbbsmmbbmmbbsm"] as const;

const WHITE = { r: 1, g: 1, b: 1 };
const GOLD = { r: 1, g: 0.78, b: 0.25 };
const RED = { r: 1, g: 0.32, b: 0.3 };
const MINT = { r: 0.6, g: 1, b: 0.75 };
const FROSTED = { r: 0.55, g: 0.8, b: 1 };

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

/**
 * Tuning, read on its first use in this call: a resource read is a call into
 * the engine that encodes every field, so a system that needs the knobs only
 * on an event does not pay for them every tick.
 */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must2(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

function must2<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A cell's centre in world units (map rows count down from the top). */
export function cellCenter(x: number, y: number): { x: number; y: number } {
  return { x: x + 0.5, y: ROWS - y - 0.5 };
}

/** The map cell under a world point. */
export function cellOf(px: number, py: number): { x: number; y: number } {
  return { x: Math.floor(px), y: ROWS - 1 - Math.floor(py) };
}

/** Whether a map cell can take a plant: the lawn inside the hedge, not a stone. */
export function buildable(world: World, g: GameV, x: number, y: number): boolean {
  if (x < 1 || x > COLS - 2 || y < 1 || y > ROWS - 2) return false;
  return og.tilemap.getTile(world, g.map as unknown as Entity, "walls", x, y) < 0;
}

/** The pads tile of each plant (tiles/garden.ogt): the map remembers what grows where. */
export const PAD = [6, 8, 9] as const;

/** The plant on a cell by its pads tile (Plant.*), or -1. */
export function padOf(world: World, g: GameV, x: number, y: number): number {
  return (PAD as readonly number[]).indexOf(og.tilemap.getTile(world, g.map as unknown as Entity, "pads", x, y));
}

export function cost(t: TuneV, kind: number): number {
  return kind === Plant.Thorn ? t.thornCost : kind === Plant.Frost ? t.frostCost : t.hedgeCost;
}

export function range(t: TuneV, kind: number): number {
  return kind === Plant.Thorn ? t.thornRange : kind === Plant.Frost ? t.frostRange : 0;
}

function occupied(g: GameV, x: number, y: number): boolean {
  const i = y * COLS + x;
  const word = [g.occ0, g.occ1, g.occ2, g.occ3][i >> 5]!;
  return (word & (1 << (i & 31))) !== 0;
}

function float(world: World, text: string, x: number, y: number, color: { r: number; g: number; b: number }, crit = false, size = 0.42): void {
  must(og.render.floatingText(world, { text, position: { x, y, z: 0 }, size, color, crit, rise: 1, lifetime: 0.8 }));
}

/**
 * Sets the selected plant on a cell, or refuses with the reason: too few
 * coins, the cell taken, a visitor standing there, or a path closed (the
 * gate's to the doorstep or any visitor's). The grid's cost is set first and
 * put back when the path check fails.
 */
export function place(world: World, g: GameV, t: TuneV, x: number, y: number): boolean {
  const refuse = (why: number, text: string) => {
    g.refused += 1;
    g.refusal = why;
    g.refusedTick = world.tick;
    const c = cellCenter(x, y);
    float(world, text, c.x, c.y + 0.6, RED, false, 0.36);
    return false;
  };
  if (!buildable(world, g, x, y) || padOf(world, g, x, y) >= 0 || occupied(g, x, y)) return refuse(Refusal.Taken, say(world, "float.taken"));
  const price = cost(t, g.selected);
  if (g.gold < price) return refuse(Refusal.Poor, say(world, "float.poor", { n: price - g.gold }));
  const grid = g.grid as unknown as never;
  const navY = ROWS - 1 - y;
  must(og.nav.setCost(world, grid, x, navY, 0));
  if (!pathsOpen(world, g)) {
    must(og.nav.setCost(world, grid, x, navY, 1));
    return refuse(Refusal.Blocked, say(world, "float.blocked"));
  }
  g.gold -= price;
  g.placed += 1;
  // The pads layer holds which plant stands on each cell; the Plants system switches the cell's row on.
  must(og.tilemap.setTile(world, { entity: g.map as unknown as Entity, layer: "pads", x, y, tile: PAD[g.selected]! }));
  g.pendingPlants += 1;
  g.sfx |= Sound.Plant;
  return true;
}

/** Whether the gate and every cell a visitor stands on still reach the doorstep. */
function pathsOpen(world: World, g: GameV): boolean {
  const grid = g.grid as unknown as never;
  const goal = cellCenter(DOOR.x, DOOR.y);
  const to = { x: goal.x, y: goal.y, z: 0 };
  const reaches = (cx: number, cy: number) => {
    const from = cellCenter(cx, cy);
    const p = og.nav.findPath(world, { nav: grid, from: { x: from.x, y: from.y, z: 0 }, to, smooth: false });
    return p.ok && p.value.complete;
  };
  if (!reaches(GATE.x, GATE.y)) return false;
  for (let i = 0; i < COLS * ROWS; i++) if (occupied(g, i % COLS, Math.floor(i / COLS)) && !reaches(i % COLS, Math.floor(i / COLS))) return false;
  return true;
}

/** The world point under a CSS pixel of the play surface, through the camera Frame wrote. */
export function screenToWorld(world: World, px: number, py: number): { x: number; y: number } | null {
  const cam = world.resource(og.Camera);
  const vp = og.ui.viewport(world);
  if (!cam.ok || vp.height <= 0) return null;
  const scale = vp.height / cam.value.height;
  return { x: cam.value.position.x + (px - vp.width / 2) / scale, y: cam.value.position.y - (py - vp.height / 2) / scale };
}

/**
 * The player's hands: the cursor (arrows, WASD, the pad's d-pad), the tray
 * (1, 2, 3, the pad's X), setting a plant (Space, Enter, the pad's A, or a
 * tap on the lawn, which also moves the cursor), the next wave (N, the
 * pad's Y) and pause (Escape, P, the pad's Start). Every input is answered
 * on its tick: the cursor moves with a tick of sound, a plant pops up, a
 * refusal says why.
 */
export const Control = defineSystem(
  "Control",
  { reads: [og.Input, og.Gamepad, og.Camera, Tuning], writes: [Game, Cursor, og.Transform] },
  (q, time, world) => {
    const game = world.resource(Game);
    const input = world.resource(og.Input);
    const pad = world.resource(og.Gamepad);
    if (!game.ok || !input.ok || !pad.ok) return;
    const g = game.value;
    const t = lazyTuning(world);
    const inp = input.value;
    const gp = pad.value;
    const playing = g.phase === Phase.Play && !g.paused && !g.settingsOpen;
    for (const a of q.archetypes()) {
      const cur = a.column(Cursor);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        if (!playing) {
          p[i * 3] = PARK.x;
          p[i * 3 + 1] = PARK.y;
          // Focus moving over a menu ticks (og-ui plays nothing when keyboard or pad focus moves).
          const nav = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab"].some((k) => inp.pressed(k)) || (["up", "down", "left", "right"] as const).some((b) => gp.pressed(b));
          if (nav) g.sfx |= Sound.Move;
          continue;
        }
        let dx = 0;
        let dy = 0;
        if (inp.pressed("ArrowLeft") || inp.pressed("KeyA") || gp.pressed("left")) dx -= 1;
        if (inp.pressed("ArrowRight") || inp.pressed("KeyD") || gp.pressed("right")) dx += 1;
        if (inp.pressed("ArrowUp") || inp.pressed("KeyW") || gp.pressed("up")) dy -= 1;
        if (inp.pressed("ArrowDown") || inp.pressed("KeyS") || gp.pressed("down")) dy += 1;
        if (dx !== 0 || dy !== 0) {
          g.cursorX = Math.max(1, Math.min(COLS - 2, g.cursorX + dx));
          g.cursorY = Math.max(1, Math.min(ROWS - 2, g.cursorY + dy));
          cur.moves[i] = cur.moves[i]! + 1;
          g.sfx |= Sound.Move;
        }
        if (inp.pressed("Digit1")) select(g, Plant.Thorn);
        if (inp.pressed("Digit2")) select(g, Plant.Frost);
        if (inp.pressed("Digit3")) select(g, Plant.Hedge);
        if (gp.pressed("x")) select(g, (g.selected + 1) % PLANTS.length);
        if (inp.pressed("KeyN") || gp.pressed("y")) g.callNext = true;
        if (inp.pressed("Escape") || inp.pressed("KeyP") || gp.pressed("start")) g.paused = true;
        if (inp.pressed("Space") || inp.pressed("Enter") || gp.pressed("a")) place(world, g, t, g.cursorX, g.cursorY);
        // A tap on the lawn moves the cursor there and plants.
        if (inp.buttonPressed("left")) {
          const at = screenToWorld(world, inp.pointer.x, inp.pointer.y);
          if (at !== null && at.x >= 0 && at.x < COLS && at.y >= 0 && at.y < ROWS) {
            const c = cellOf(at.x, at.y);
            g.cursorX = c.x;
            g.cursorY = c.y;
            place(world, g, t, c.x, c.y);
          }
        }
        const c = cellCenter(g.cursorX, g.cursorY);
        p[i * 3] = c.x;
        p[i * 3 + 1] = c.y;
        // The cursor and, for a shooting plant, its range: drawn this tick.
        const ok = buildable(world, g, g.cursorX, g.cursorY) && g.gold >= cost(t, g.selected);
        const frame = ok ? 18 : 19;
        const sheet = g.sheet as unknown as never;
        must(og.render.sprite(world, { texture: sheet, position: { x: c.x, y: c.y, z: 0 }, width: 1, height: 1, uv: [(frame % 8) / 8, Math.floor(frame / 8) / 3, (frame % 8 + 1) / 8, (Math.floor(frame / 8) + 1) / 3], layer: 8 }));
        const r = range(t, g.selected);
        if (ok && r > 0) must(og.render.sprite(world, { texture: g.ring as unknown as never, position: { x: c.x, y: c.y, z: 0 }, width: r * 2, height: r * 2, layer: 7, opacity: 0.85 }));
      }
    }
    void time;
  },
);

function select(g: GameV, kind: number): void {
  if (g.selected !== kind) g.sfx |= Sound.Move;
  g.selected = kind;
}

/** Plants: reload, sight from the tunables, the shooting clip back to the sway. Held by a hit stop. */
export const Plants = defineSystem(
  "Plants",
  { reads: [Tuning], writes: [Game, Bed, og.Perception, og.Flipbook, og.Sprite, og.AiAgent], after: ["Control"] },
  (q, time, world) => {
    const game = world.resource(Game);
    if (!game.ok) return;
    const t = lazyTuning(world);
    // Sight follows the tunables twice a second (Tune writes them live).
    const refresh = world.tick % 30 === 0;
    const g = game.value;
    const held = time.hitStop > 0 || g.paused || g.phase !== Phase.Play;
    const reset = g.reset;
    const pending = g.pendingPlants > 0;
    if (pending) g.pendingPlants = 0;
    for (const a of q.archetypes()) {
      const b = a.column(Bed);
      const per = a.column(og.Perception);
      const fb = a.column(og.Flipbook);
      const sprite = a.column(og.Sprite);
      const ai = a.column(og.AiAgent);
      for (let i = 0; i < a.count; i++) {
        if (reset) {
          b.planted[i] = 0;
          b.cool[i] = 0;
          b.anim[i] = 0;
          b.shots[i] = 0;
          sprite.visible[i] = 0;
          ai.active[i] = 0;
          per.sight[i] = 0;
          continue;
        }
        if (pending && !b.planted[i]) {
          // A plant set this tick: its cell's pads tile names it.
          const kind = padOf(world, g, b.cellX[i]!, b.cellY[i]!);
          if (kind >= 0) {
            b.planted[i] = 1;
            b.kind[i] = kind;
            b.cool[i] = 20;
            b.anim[i] = 0;
            sprite.visible[i] = 1;
            fb.clip[i] = clipOf(PLANTS[kind]!);
            ai.active[i] = kind === Plant.Hedge ? 0 : 1;
            per.sight[i] = range(t, kind);
            must(og.tween.squash(world, { entity: a.entity(i), impact: { x: 0.6, y: 1.5 }, duration: 0.25 }));
          }
        }
        if (!b.planted[i]) continue;
        if (refresh) per.sight[i] = range(t, b.kind[i]!);
        if (held) continue;
        if (b.cool[i]! > 0) b.cool[i] = b.cool[i]! - 1;
        if (b.anim[i]! > 0) {
          b.anim[i] = b.anim[i]! - 1;
          if (b.anim[i] === 0) fb.clip[i] = clipOf(PLANTS[b.kind[i]!]!);
        }
      }
    }
  },
);

/** Tween tags of shots: (kind + 1) × 1000 + the visitor's slot. */
const TAG = 1000;

/**
 * `fire:<plant>`: the behaviour's shot (og.ai queues it with the plant's
 * handle). It sends a free shot from the plant to where its target will be
 * when the shot lands, on a tween whose tag names the target and the kind.
 */
export function fire(world: World, args: readonly string[]): void {
  const me = Number(args[0]) as Entity;
  const game = world.resource(Game);
  const tune = world.resource(Tuning);
  const bed = world.get(me, Bed);
  const eyes = world.get(me, og.Perception);
  const at = world.get(me, og.Transform);
  if (!game.ok || !tune.ok || !bed.ok || !eyes.ok || !at.ok) throw new Error(`fire: ${args[0]} is not a plant`);
  const g = game.value;
  const t = tune.value;
  const b = bed.value;
  if (!b.planted || b.cool > 0 || !eyes.value.sees || g.phase !== Phase.Play || g.paused) return;
  const target = eyes.value.target as Entity;
  const v = world.get(target, Visitor);
  const tp = world.get(target, og.Transform);
  const nav = world.get(target, og.NavAgent);
  if (!v.ok || !tp.ok || !nav.ok || !v.value.active || v.value.dying > 0) return;
  let shot: Entity | null = null;
  for (let k = 0; k < SHOTS && shot === null; k++) {
    const e = world.find(`shot-${(g.shots + k) % SHOTS}`);
    if (e !== null) {
      const s = world.get(e, Shot);
      if (s.ok && (s.value.tween === 0 || !og.tween.active(world, s.value.tween as unknown as never))) shot = e;
    }
  }
  if (shot === null) return;
  const from = { x: at.value.position.x, y: at.value.position.y + 0.2 };
  let to = { x: tp.value.position.x, y: tp.value.position.y };
  const flight = Math.min(0.5, Math.hypot(to.x - from.x, to.y - from.y) / t.shotSpeed);
  // Lead the target by where it walks during the flight.
  to = { x: to.x + nav.value.velocity.x * flight, y: to.y + nav.value.velocity.y * flight };
  const kind = b.kind;
  must(world.set(shot, og.Transform, { position: { x: from.x, y: from.y, z: 0 } }));
  must(world.set(shot, og.Sprite, { opacity: 1, visible: true }));
  must(world.set(shot, og.Flipbook, { clip: clipOf(kind === Plant.Frost ? "flake" : "seed") }));
  const flightTween = must2(
    og.tween.sequence(
      world,
      [
        { entity: shot, field: "position", from: [from.x, from.y, 0], to: [to.x, to.y, 0], duration: flight, ease: "linear" },
        { entity: shot, component: og.Sprite, field: "opacity", to: [0], duration: 0.05, ease: "linear" },
      ],
      { tag: (kind + 1) * TAG + v.value.slot },
    ),
  );
  must(world.set(shot, Shot, { tween: flightTween as unknown as number }));
  must(world.set(me, Bed, { cool: kind === Plant.Frost ? t.frostReload : t.thornReload, anim: 12, shots: b.shots + 1 }));
  must(world.set(me, og.Flipbook, { clip: clipOf(`${PLANTS[kind]}_shoot`) }));
  must(og.tween.squash(world, { entity: me, impact: { x: 1.15, y: 0.85 }, duration: 0.15 }));
  g.shots += 1;
  g.sfx |= kind === Plant.Frost ? Sound.Frost : Sound.Thorn;
}

/**
 * The visitors and the waves: a wave starts after the break (or at once,
 * on the player's call, which pays a bonus for the seconds saved), its
 * visitors come through the gate one gap apart and walk the flow field;
 * shots land by their tween's tag; a visitor at 0 health pops (a coin
 * count, a burst of petals, a light hit stop at most every 10 ticks); one
 * that reaches the doorstep takes its light (shake, hit stop, a red
 * number). The last visitor of the last wave gone ends the round as won;
 * the light at 0 ends it lost.
 */
export const Visitors = defineSystem(
  "Visitors",
  {
    reads: [Tuning, og.TweenCompleted],
    writes: [Game, Visitor, og.Transform, og.Sprite, og.NavAgent, og.Flipbook, og.ParticleSystem],
    after: ["Plants"],
  },
  (q, time, world) => {
    const game = world.resource(Game);
    if (!game.ok) return;
    const g = game.value;
    const t = lazyTuning(world);
    // Shots that landed this tick: damage and frost by the visitor's slot.
    const damage = new Array<number>(VISITORS).fill(0);
    const frost = new Array<boolean>(VISITORS).fill(false);
    for (const e of world.events(og.TweenCompleted)) {
      if (e.tag < TAG) continue;
      const kind = Math.floor(e.tag / TAG) - 1;
      const slot = e.tag % TAG;
      if (slot < VISITORS) {
        damage[slot] = damage[slot]! + (kind === Plant.Frost ? t.frostDamage : t.thornDamage);
        if (kind === Plant.Frost) frost[slot] = true;
      }
    }
    const playing = g.phase === Phase.Play && !g.paused;
    const held = time.hitStop > 0 || !playing;

    // The wave clock.
    let spawnKind = -1;
    if (playing && time.hitStop === 0) {
      if (!g.inWave && g.wave < WAVES.length) {
        if (g.callNext && g.breakTicks > 0) {
          const bonus = Math.ceil(g.breakTicks / 60) * 2;
          g.gold += bonus;
          g.breakTicks = 0;
        }
        if (g.breakTicks > 0) g.breakTicks -= 1;
        if (g.breakTicks === 0) startWave(world, g);
      } else if (g.inWave && g.toSpawn > 0) {
        if (g.spawnTicks > 0) g.spawnTicks -= 1;
        if (g.spawnTicks === 0) {
          const letters = WAVES[g.wave - 1]!;
          spawnKind = "bms".indexOf(letters[g.spawned]!);
          g.spawned += 1;
          g.toSpawn -= 1;
          g.spawnTicks = Math.max(1, Math.round(t.spawnGap * 60));
        }
      }
    }
    if (g.callNext) g.callNext = false;

    let walking = 0;
    const occ = [0, 0, 0, 0];
    const goal = cellCenter(DOOR.x, DOOR.y);
    const gate = cellCenter(GATE.x, GATE.y);
    for (const a of q.archetypes()) {
      const v = a.column(Visitor);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      const nav = a.column(og.NavAgent);
      const fb = a.column(og.Flipbook);
      const fx = a.column(og.ParticleSystem);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        if (g.reset) {
          v.active[i] = 0;
          v.dying[i] = 0;
          sprite.visible[i] = 0;
          park(p, nav, fx, i);
          continue;
        }
        if (v.dying[i]! > 0) {
          if (!held) v.dying[i] = v.dying[i]! - 1;
          if (v.dying[i] === 0) park(p, nav, fx, i);
          continue;
        }
        if (!v.active[i]) {
          if (spawnKind >= 0) {
            // Through the gate: this row is the next visitor.
            const k = KINDS[spawnKind]!;
            const hp = Math.round(k.hp * (1 + t.hpGrowth * (g.wave - 1)));
            v.active[i] = 1;
            v.kind[i] = spawnKind;
            v.hp[i] = hp;
            v.maxHp[i] = hp;
            v.speed[i] = k.speed;
            v.slow[i] = 0;
            p[i * 3] = gate.x;
            p[i * 3 + 1] = gate.y + 0.3;
            sprite.visible[i] = 1;
            sprite.opacity[i] = 1;
            sprite.color[i * 3] = 1;
            sprite.color[i * 3 + 1] = 1;
            sprite.color[i * 3 + 2] = 1;
            fb.clip[i] = clipOf(k.name);
            nav.seeking[i] = 1;
            nav.speed[i] = k.speed;
            spawnKind = -1;
          }
          continue;
        }
        if (damage[v.slot[i]!]! > 0 && !held) {
          v.hp[i] = Math.max(0, v.hp[i]! - damage[v.slot[i]!]!);
          if (frost[v.slot[i]!]) v.slow[i] = t.frostTicks;
          must(og.tween.flash(world, { entity: me, duration: 0.08 }));
        }
        const x = p[i * 3]!;
        const y = p[i * 3 + 1]!;
        if (v.hp[i]! <= 0) {
          pop(world, g, t, v, sprite, nav, fx, i, x, y);
          continue;
        }
        if (Math.hypot(x - goal.x, y - goal.y) < 0.45 || nav.status[i] === NAV_ARRIVED) {
          leak(world, g, t, v, i);
          park(p, nav, fx, i);
          sprite.visible[i] = 0;
          continue;
        }
        if (!held && v.slow[i]! > 0) v.slow[i] = v.slow[i]! - 1;
        const slowed = v.slow[i]! > 0;
        nav.speed[i] = v.speed[i]! * (slowed ? t.frostSlow : 1);
        fb.speed[i] = slowed ? t.frostSlow : 1;
        const tint = slowed ? FROSTED : WHITE;
        sprite.color[i * 3] = tint.r;
        sprite.color[i * 3 + 1] = tint.g;
        sprite.color[i * 3 + 2] = tint.b;
        walking += 1;
        const c = cellOf(x, y);
        if (c.x >= 0 && c.x < COLS && c.y >= 0 && c.y < ROWS) {
          const idx = c.y * COLS + c.x;
          occ[idx >> 5] = (occ[idx >> 5]! | (1 << (idx & 31))) >>> 0;
        }
        // A health bar over a hurt visitor.
        if (v.hp[i]! < v.maxHp[i]!) {
          const f = v.hp[i]! / v.maxHp[i]!;
          must(og.render.sprite(world, { position: { x: x - 0.35, y: y + 0.5, z: 0 }, width: 0.7, height: 0.1, anchor: { x: 0, y: 0.5 }, color: { r: 0.05, g: 0.08, b: 0.06 }, layer: 9 }));
          must(og.render.sprite(world, { position: { x: x - 0.35, y: y + 0.5, z: 0 }, width: 0.7 * f, height: 0.1, anchor: { x: 0, y: 0.5 }, color: f > 0.5 ? { r: 0.35, g: 0.9, b: 0.45 } : { r: 1, g: 0.55, b: 0.2 }, layer: 9 }));
        }
      }
    }
    // A resource field written is a call into the engine: write only what changed.
    if (g.reset) g.reset = false;
    if (occ[0] !== g.occ0) g.occ0 = occ[0]!;
    if (occ[1] !== g.occ1) g.occ1 = occ[1]!;
    if (occ[2] !== g.occ2) g.occ2 = occ[2]!;
    if (occ[3] !== g.occ3) g.occ3 = occ[3]!;
    if (walking !== g.walking) g.walking = walking;

    if (!playing) return;
    if (g.lives === 0) {
      endRound(world, g, false);
      return;
    }
    if (g.inWave && g.toSpawn === 0 && walking === 0 && time.hitStop === 0) {
      g.inWave = false;
      const bonus = 20 + 5 * g.wave;
      g.gold += bonus;
      g.sfx |= Sound.Clear;
      float(world, say(world, "float.clear", { n: bonus }), goal.x, goal.y + 2.2, MINT, true, 0.45);
      if (g.wave >= WAVES.length) endRound(world, g, true);
      else g.breakTicks = Math.max(1, Math.round(t.waveBreak * 60));
    }
  },
);

function startWave(world: World, g: GameV): void {
  g.wave += 1;
  g.inWave = true;
  g.toSpawn = WAVES[g.wave - 1]!.length;
  g.spawned = 0;
  g.spawnTicks = 1;
  g.sfx |= Sound.Wave;
  const gate = cellCenter(GATE.x, GATE.y);
  float(world, say(world, "float.wave", { n: g.wave }), gate.x, gate.y - 1.2, GOLD, true, 0.5);
}

function park(p: Float32Array, nav: View<typeof og.NavAgent>, fx: View<typeof og.ParticleSystem>, i: number): void {
  p[i * 3] = PARK.x;
  p[i * 3 + 1] = PARK.y;
  nav.seeking[i] = 0;
  fx.visible[i] = 0;
}

function pop(world: World, g: GameV, t: TuneV, v: View<typeof Visitor>, sprite: View<typeof og.Sprite>, nav: View<typeof og.NavAgent>, fx: View<typeof og.ParticleSystem>, i: number, x: number, y: number): void {
  const k = KINDS[v.kind[i]!]!;
  v.active[i] = 0;
  v.dying[i] = POP_TICKS;
  sprite.visible[i] = 0;
  nav.seeking[i] = 0;
  fx.visible[i] = 1;
  fx.startTick[i] = world.tick;
  g.kills += 1;
  g.gold += k.bounty;
  g.sfx |= Sound.Pop;
  float(world, say(world, "float.gold", { n: k.bounty }), x, y + 0.4, GOLD);
  // A light hit stop, at most one per 10 ticks so a stream of pops does not stutter (og-feel §3).
  if (world.tick - g.lastStop >= 10 && t.killStop > 0) {
    must(og.time.hitStop(world, t.killStop));
    g.lastStop = world.tick;
  }
}

function leak(world: World, g: GameV, t: TuneV, v: View<typeof Visitor>, i: number): void {
  const k = KINDS[v.kind[i]!]!;
  v.active[i] = 0;
  g.leaks += 1;
  g.lives = Math.max(0, g.lives - k.harm);
  g.sfx |= Sound.Leak;
  const goal = cellCenter(DOOR.x, DOOR.y);
  float(world, say(world, "float.leak", { n: k.harm }), goal.x, goal.y + 0.8, RED, true, 0.5);
  must(og.time.hitStop(world, t.leakStop));
  must(og.camera.shake(world, t.leakShake));
  must(og.camera.kick(world, { x: 0, y: -1, z: 0 }, 0.15));
  const lantern = world.find("lantern");
  if (lantern !== null) {
    must(og.tween.flash(world, { entity: lantern, color: RED, duration: 0.25 }));
    must(og.tween.squash(world, { entity: lantern, impact: { x: 1.25, y: 0.8 }, duration: 0.25 }));
  }
}

/** The round is over: the score, the best (saved with its revision), the board and the telemetry. */
export function endRound(world: World, g: GameV, won: boolean): void {
  g.phase = Phase.Over;
  g.won = won;
  g.inWave = false;
  g.overTick = world.tick;
  g.score = g.kills * 10 + g.lives * 50 + g.gold;
  g.newBest = g.score > g.best;
  g.sfx |= won ? Sound.Won : Sound.Lost;
  if (g.newBest) {
    g.best = g.score;
    must(og.platform.storeSet(world, { key: "best", value: g.score, revision: g.bestRevision }));
  }
  must(og.platform.leaderboardSubmit(world, { board: "main", score: g.score }));
  must(og.platform.telemetry(world, "round.over", { won, wave: g.wave, kills: g.kills, lives: g.lives, score: g.score }));
}

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Visitors"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.sfx === 0) return;
  const rng = world.resource(og.Rng);
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((g.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        // Shots and pops vary a little, so a stream of them does not machine-gun (og-audio §1).
        if ((b === Sound.Thorn || b === Sound.Pop || b === Sound.Frost) && rng.ok) src.pitch[i] = rng.value.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
