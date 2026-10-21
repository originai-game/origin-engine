// What Wick meets: ticks (clockwork beetles walking their floor; a stomp
// from above stops one with a hit stop, a shake, sparks and a bounce; any
// other touch hurts), gears (taken by touch: a sound, sparks, a number),
// lamps (the checkpoints: lit as he passes, the crates' places recorded and
// the world saved), the exit hatch (it opens, he cheers, the results
// follow) and spikes. Each works through its own archetype views and the
// Game fields Control wrote (heroX, heroY, heroVY).

import { og, defineSystem } from "@origingame/engine";
import { Exit, Fx, FxKind, Game, Gear, Hazard, Lamp, Member, Phase, Sfx, Sound, Tick, Tuning } from "./components.ts";
import { effect } from "./feel.ts";
import { hurtWick } from "./hero.ts";

/** Wick's box from his feet: 0.8 wide, 1.5 tall. */
const HALF_W = 0.4;
const TALL = 1.5;

export const Ticks = defineSystem("Ticks", { reads: [Tuning, Member], writes: [Game, Sfx, Fx, Tick, og.Transform, og.Sprite, og.Flipbook] }, (q, time, world) => {
  const game = world.resource(Game);
  const tuning = world.resource(Tuning);
  const sfx = world.resource(Sfx);
  const fx = world.resource(Fx);
  if (!game.ok || !tuning.ok || !sfx.ok || !fx.ok) return;
  const g = game.value;
  const speed = tuning.value.tickSpeed;
  let hurt = false;
  let stomped: [number, number] | null = null;
  for (const a of q.archetypes()) {
    const t = a.column(Tick);
    const level = a.column(Member).level;
    const pos = a.column(og.Transform).position;
    const sprite = a.column(og.Sprite);
    const fb = a.column(og.Flipbook);
    for (let i = 0; i < a.count; i++) {
      if (g.reset === 2 && level[i] === g.resetLevel) {
        pos[i * 3] = t.homeX[i]!;
        t.dir[i] = 1;
        t.dead[i] = 0;
        t.deadTicks[i] = 0;
        sprite.visible[i] = 1;
        sprite.opacity[i] = 1;
        fb.clip[i] = 0;
      }
      if (t.dead[i]) {
        // A stopped tick lies squashed a moment, then fades out.
        if (t.deadTicks[i]! < 60) t.deadTicks[i] = t.deadTicks[i]! + 1;
        sprite.opacity[i] = Math.max(0, 1 - Math.max(0, t.deadTicks[i]! - 30) / 30);
        if (t.deadTicks[i] === 60) sprite.visible[i] = 0;
        continue;
      }
      let x = pos[i * 3]! + t.dir[i]! * speed * time.scaledDt;
      if (x > t.maxX[i]!) {
        x = t.maxX[i]!;
        t.dir[i] = -1;
      } else if (x < t.minX[i]!) {
        x = t.minX[i]!;
        t.dir[i] = 1;
      }
      pos[i * 3] = x;
      sprite.flipX[i] = t.dir[i]! < 0 ? 1 : 0;
      if (g.phase !== Phase.Play || level[i] !== g.level || g.hurt > 0) continue;
      const y = pos[i * 3 + 1]!;
      const overlaps = Math.abs(g.heroX - x) < HALF_W + 0.6 && g.heroY < y + 0.8 && g.heroY + TALL > y;
      if (!overlaps) continue;
      if (g.heroVY < 0 && g.heroY > y + 0.35) {
        t.dead[i] = 1;
        t.deadTicks[i] = 0;
        fb.clip[i] = 1;
        stomped = [x, y + 0.4];
      } else if (g.safe === 0) hurt = true;
    }
  }
  if (stomped !== null) {
    g.stomps += 1;
    g.bounce = true;
    sfx.value.bits |= Sound.Stomp;
    effect(fx.value, FxKind.Stomp, stomped[0], stomped[1], time.tick);
    og.time.hitStop(world, 4);
    og.camera.shake(world, 0.18);
    og.render.floatingText(world, { text: "!", position: { x: stomped[0], y: stomped[1] + 0.8, z: 0 }, color: { r: 1, g: 0.85, b: 0.4 } });
  } else if (hurt) hurtWick(g);
});

export const Pickups = defineSystem("Pickups", { reads: [Member], writes: [Game, Sfx, Fx, Gear, og.Transform, og.Sprite] }, (q, time, world) => {
  const game = world.resource(Game);
  const sfx = world.resource(Sfx);
  const fx = world.resource(Fx);
  if (!game.ok || !sfx.ok || !fx.ok) return;
  const g = game.value;
  const cx = g.heroX;
  const cy = g.heroY + 0.75;
  let taken: [number, number] | null = null;
  for (const a of q.archetypes()) {
    const gear = a.column(Gear);
    const level = a.column(Member).level;
    const pos = a.column(og.Transform).position;
    const vis = a.column(og.Sprite).visible;
    for (let i = 0; i < a.count; i++) {
      if (g.reset === 2 && level[i] === g.resetLevel) {
        gear.taken[i] = 0;
        vis[i] = 1;
      }
      if (gear.taken[i]) continue;
      // A slow bob (scaled time: it holds in a pause and a hit stop).
      const y = gear.baseY[i]! + Math.sin((time.scaledElapsed * 0.6 + gear.phase[i]!) * Math.PI * 2) * 0.12;
      pos[i * 3 + 1] = y;
      if (g.phase !== Phase.Play || level[i] !== g.level || g.hurt > 0) continue;
      if (Math.abs(pos[i * 3]! - cx) < 0.85 && Math.abs(y - cy) < 1.1) {
        gear.taken[i] = 1;
        vis[i] = 0;
        g.gears += 1;
        g.gearMask |= 1 << gear.index[i]!;
        taken = [pos[i * 3]!, y];
      }
    }
  }
  if (taken !== null) {
    sfx.value.bits |= Sound.Gear;
    effect(fx.value, FxKind.Gear, taken[0], taken[1], time.tick);
    og.render.floatingText(world, { text: "+1", position: { x: taken[0], y: taken[1] + 0.5, z: 0 }, color: { r: 1, g: 0.85, b: 0.35 } });
  }
});

/** Lamps light as Wick passes: the checkpoint moves there, the crates' places are recorded and the world is saved. */
export const Lamps = defineSystem("Lamps", { reads: [Member], writes: [Game, Sfx, Lamp, og.Flipbook, og.Light2D, og.Sprite, og.Transform] }, (q, _time, world) => {
  const game = world.resource(Game);
  const sfx = world.resource(Sfx);
  if (!game.ok || !sfx.ok) return;
  const g = game.value;
  for (const a of q.archetypes()) {
    const lamp = a.column(Lamp);
    const level = a.column(Member).level;
    const pos = a.column(og.Transform).position;
    const fb = a.column(og.Flipbook).clip;
    const light = a.column(og.Light2D).intensity;
    const glow = a.column(og.Sprite).glow;
    for (let i = 0; i < a.count; i++) {
      if (g.reset === 2 && level[i] === g.resetLevel) {
        lamp.lit[i] = 0;
        fb[i] = 0;
        light[i] = 0;
        glow[i] = 0;
      }
      if (lamp.lit[i] || g.phase !== Phase.Play || level[i] !== g.level || g.hurt > 0) continue;
      if (Math.abs(pos[i * 3]! - g.heroX) < 1 && Math.abs(pos[i * 3 + 1]! - g.heroY) < 1.5) {
        lamp.lit[i] = 1;
        fb[i] = 1;
        light[i] = 1.7;
        glow[i] = 1.2;
        g.checkpoint = lamp.index[i]!;
        g.checkX = pos[i * 3]!;
        g.checkY = pos[i * 3 + 1]!;
        g.lampsLit += 1;
        g.snap = true;
        g.saveNow = true;
        sfx.value.bits |= Sound.Lamp;
      }
    }
  }
});

/** The exit hatch opens when Wick reaches it; the cheer runs Game.exiting up to the results (src/ui.ts). */
export const Exits = defineSystem("Exits", { reads: [Member, Tuning], writes: [Game, Sfx, Exit, og.Flipbook, og.Transform] }, (q, _time, world) => {
  const game = world.resource(Game);
  const sfx = world.resource(Sfx);
  if (!game.ok || !sfx.ok) return;
  const g = game.value;
  if (g.exiting > 0) g.exiting += 1;
  for (const a of q.archetypes()) {
    const exit = a.column(Exit);
    const level = a.column(Member).level;
    const pos = a.column(og.Transform).position;
    const fb = a.column(og.Flipbook);
    for (let i = 0; i < a.count; i++) {
      if (g.reset === 2 && level[i] === g.resetLevel) {
        exit.open[i] = 0;
        fb.clip[i] = 0;
      }
      if (exit.open[i] || g.phase !== Phase.Play || level[i] !== g.level || g.hurt > 0) continue;
      if (Math.abs(pos[i * 3]! - g.heroX) < 0.9 && Math.abs(pos[i * 3 + 1]! - g.heroY) < 1) {
        exit.open[i] = 1;
        fb.clip[i] = 1;
        fb.time[i] = 0;
        g.exiting = 1;
        sfx.value.bits |= Sound.Exit;
      }
    }
  }
});

export const Hazards = defineSystem("Hazards", { reads: [Hazard, Member], writes: [Game, Sfx] }, (q, time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.phase !== Phase.Play || g.hurt > 0 || g.safe > 0) return;
  let hit = false;
  for (const a of q.archetypes()) {
    const h = a.column(Hazard);
    const level = a.column(Member).level;
    for (let i = 0; i < a.count; i++) {
      if (level[i] !== g.level) continue;
      if (g.heroX + HALF_W > h.x0[i]! && g.heroX - HALF_W < h.x1[i]! && g.heroY < h.y1[i]! && g.heroY + TALL > h.y0[i]!) hit = true;
    }
  }
  if (hit) hurtWick(g);
});
