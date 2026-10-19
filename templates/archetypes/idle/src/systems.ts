// Spark Orb's systems, in tick order after the screens (src/ui.ts): Orb
// (taps: the pointer on the orb, Space, Enter, the pad's A, the HUD), the
// Economy (what the generators make, purchases, milestones, the
// supernova), Motes (one mote circling the orb per generator owned),
// Bursts (the particle bursts the others ask for) and Voices.
//
// A system declares the components its rows carry; what another system's
// rows must show (a burst, a mote) is asked for through the Game resource
// and done by the system that owns those rows, the same tick.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Burst, Game, GENERATORS, Mote, MOTES, Orb, Phase, Sfx, Sound, Tuning } from "./components.ts";
import { big, generatorRate, novaGain, owned, price, setOwned, tapValue, totalRate } from "./numbers.ts";
import { say } from "./ui.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

const GOLD = { r: 1, g: 0.78, b: 0.3 };
const WHITE = { r: 1, g: 0.97, b: 0.9 };
/** Each generator's colour: the motes, the floating words. */
export const TIER_COLOURS = [
  { r: 1, g: 0.55, b: 0.2 },
  { r: 0.45, g: 0.8, b: 1 },
  { r: 0.8, g: 0.55, b: 1 },
  { r: 1, g: 0.95, b: 0.75 },
] as const;
/** The orb's radius in world units at scale 1. */
export const ORB_RADIUS = 1.35;
/** Burst kinds (Burst.kind, Game.burstKind). */
export const BURST = { Tap: 0, Milestone: 1, Nova: 2 } as const;

export function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

export function must2<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Tuning, read on its first use in this call (a resource read encodes every field). */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must2(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

export function counts(g: GameV): number[] {
  return [g.owned0, g.owned1, g.owned2, g.owned3];
}

function float(world: World, text: string, x: number, y: number, color: { r: number; g: number; b: number }, crit = false, size = 0.42): void {
  must(og.render.floatingText(world, { text, position: { x, y, z: 0 }, size, color, crit, rise: 1.1, lifetime: 0.8 }));
}

/** Asks the Bursts system for a burst at a world point this tick. */
function burst(world: World, g: GameV, kind: number, x: number, y: number): void {
  g.burstKind = kind;
  g.burstX = x;
  g.burstY = y;
  g.burstTick = world.tick;
}

/** The world point under a CSS pixel, through the orthographic camera. */
export function screenToWorld(world: World, px: number, py: number): { x: number; y: number } | null {
  const cam = world.resource(og.Camera);
  const vp = og.ui.viewport(world);
  if (!cam.ok || vp.height <= 0) return null;
  const scale = vp.height / cam.value.height;
  const at = cam.value.position;
  return { x: at.x + (px - vp.width / 2) / scale, y: at.y - (py - vp.height / 2) / scale };
}

/**
 * Taps: a press on the orb, Space or Enter, the pad's A or the HUD's ask
 * each make the tap's sparks, with a squash, a flash, the number rising
 * from where the tap landed, a burst of sparks and a chime whose pitch
 * climbs with a quick run of taps. The orb breathes between taps.
 */
export const OrbTap = defineSystem(
  "Orb",
  { reads: [og.Input, og.Gamepad, og.Camera, Tuning], writes: [Game, Orb, og.Transform, og.Sprite], after: ["Ui"] },
  (q, time, world) => {
    const game = world.resource(Game);
    const input = world.resource(og.Input);
    const gamepad = world.resource(og.Gamepad);
    if (!game.ok || !input.ok || !gamepad.ok) return;
    const g = game.value;
    const inp = input.value;
    const gp = gamepad.value;
    const playing = g.phase === Phase.Play && !g.held && world.tick > g.playTick;
    for (const a of q.archetypes()) {
      const orb = a.column(Orb);
      const p = a.column(og.Transform).position;
      const s = a.column(og.Transform).scale;
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        p[i * 3] = g.orbX;
        p[i * 3 + 1] = g.orbY;
        // Breathing, by the simulation clock (paused with it).
        const breath = 1 + 0.025 * Math.sin(time.elapsed * 1.6);
        s[i * 3] = breath;
        s[i * 3 + 1] = breath;
        if (!playing) continue;
        let tap = g.pendingTap || inp.pressed("Space") || inp.pressed("Enter") || gp.pressed("a");
        let at = { x: g.orbX + 0.35, y: g.orbY + 0.9 };
        if (g.pendingTap) g.pendingTap = false;
        if (inp.buttonPressed("left")) {
          const w = screenToWorld(world, inp.pointer.x, inp.pointer.y);
          if (w !== null && Math.hypot(w.x - g.orbX, w.y - g.orbY) <= ORB_RADIUS) {
            tap = true;
            at = { x: w.x, y: w.y + 0.3 };
          }
        }
        if (!tap) continue;
        const t = lazyTuning(world);
        const value = tapValue(t, g.rate, g.dust);
        g.sparks += value;
        g.runEarned += value;
        g.lifetime += value;
        g.taps += 1;
        orb.taps[i] = orb.taps[i]! + 1;
        g.sfx |= Sound.Tap;
        // A quick run of taps climbs in pitch (the Voices system reads the streak).
        g.streak = world.tick - g.lastTapTick <= 20 ? Math.min(g.streak + 1, 12) : 0;
        g.lastTapTick = world.tick;
        float(world, `+${big(world, value)}`, at.x, at.y, g.streak >= 8 ? GOLD : WHITE, g.streak >= 8);
        must(og.tween.squash(world, { entity: me, impact: { x: 1.12, y: 0.9 }, duration: 0.16 }));
        must(og.tween.flash(world, { entity: me, color: { r: 1, g: 0.95, b: 0.8 }, duration: 0.08 }));
        burst(world, g, BURST.Tap, at.x, at.y - 0.3);
      }
    }
  },
);

/**
 * The economy: the generators' sparks each tick (time stops with the
 * pause), purchases (1–4, the pad's X on the selected generator, the HUD),
 * refusals (the card shakes with the error sound), milestones (every
 * Tuning.milestone of a generator doubles it: a burst and a fanfare) and
 * the supernova (sparks and generators back to nothing, stardust for the
 * run).
 */
export const Economy = defineSystem(
  "Economy",
  { reads: [og.Input, og.Gamepad, Tuning], writes: [Game], after: ["Orb"] },
  (_q, _time, world) => {
    const game = world.resource(Game);
    const input = world.resource(og.Input);
    const gamepad = world.resource(og.Gamepad);
    if (!game.ok || !input.ok || !gamepad.ok) return;
    const g = game.value;
    const inp = input.value;
    const gp = gamepad.value;
    const t = lazyTuning(world);
    if (g.phase !== Phase.Play) return;
    const rate = totalRate(t, counts(g), g.dust);
    if (rate !== g.rate) g.rate = rate;
    const tv = tapValue(t, rate, g.dust);
    if (tv !== g.tapValue) g.tapValue = tv;
    if (g.held) return;
    g.runTicks += 1;
    if (rate > 0) {
      const made = rate / 60;
      g.sparks += made;
      g.runEarned += made;
      g.lifetime += made;
    }
    // Choosing and buying (not with the key that began play).
    if (world.tick <= g.playTick) return;
    let buy = g.pendingBuy;
    if (g.pendingBuy >= 0) g.pendingBuy = -1;
    GENERATORS.forEach((_, i) => {
      if (inp.pressed(`Digit${i + 1}`)) buy = i;
    });
    const step = (inp.pressed("ArrowRight") || gp.pressed("right") ? 1 : 0) - (inp.pressed("ArrowLeft") || gp.pressed("left") ? 1 : 0);
    if (step !== 0) {
      g.selected = (g.selected + step + GENERATORS.length) % GENERATORS.length;
      g.sfx |= Sound.Move;
    }
    if (gp.pressed("x") || inp.pressed("KeyB")) buy = g.selected;
    if (buy >= 0) purchase(world, g, t, buy);
    if (g.pendingNova) {
      g.pendingNova = false;
      nova(world, g, t);
    }
  },
);

function purchase(world: World, g: GameV, t: TuneV, i: number): void {
  const n = owned(g, i);
  const cost = price(t, i, n);
  g.selected = i;
  if (g.sparks < cost) {
    g.refused += 1;
    g.refusedSlot = i;
    g.refusedTick = world.tick;
    return;
  }
  g.sparks -= cost;
  setOwned(g, i, n + 1);
  g.buys += 1;
  g.sfx |= Sound.Buy;
  g.boughtSlot = i;
  g.boughtTick = world.tick;
  const ring = 1.75 + 0.55 * i;
  const colour = TIER_COLOURS[i]!;
  if ((n + 1) % t.milestone === 0) {
    // A milestone: this generator now makes twice as much.
    g.milestoneSlot = i;
    g.milestoneTick = world.tick;
    g.sfx |= Sound.Milestone;
    float(world, say(world, "float.milestone", { name: say(world, `gen.${GENERATORS[i]}`) }), g.orbX, g.orbY + ring + 0.3, colour, true, 0.5);
    burst(world, g, BURST.Milestone, g.orbX, g.orbY);
    must(og.camera.punch(world, 0.06));
  } else {
    float(world, `+${big(world, generatorRate(t, i, n + 1, g.dust) - generatorRate(t, i, n, g.dust))}/s`, g.orbX + ring * 0.7, g.orbY + ring * 0.7, colour, false, 0.34);
  }
}

/** The supernova: the run's stardust, then everything but the stardust and the record back to nothing. */
export function nova(world: World, g: GameV, t: TuneV): void {
  const gain = novaGain(t, g.runEarned);
  if (gain < 1) return;
  g.lastGain = gain;
  g.lastRun = g.runEarned;
  g.lastRunTicks = g.runTicks;
  g.dust += gain;
  g.novas += 1;
  g.sparks = 0;
  g.runEarned = 0;
  g.runTicks = 0;
  g.owned0 = 0;
  g.owned1 = 0;
  g.owned2 = 0;
  g.owned3 = 0;
  g.rate = 0;
  g.phase = Phase.Nova;
  g.sfx |= Sound.Nova;
  g.saveNow = true;
  burst(world, g, BURST.Nova, g.orbX, g.orbY);
  must(og.time.hitStop(world, 10));
  must(og.camera.shake(world, 0.6));
  must(og.camera.punch(world, 0.2));
  must(og.platform.telemetry(world, "orb.nova", { gain, dust: g.dust, novas: g.novas }));
}

/** The motes: generator i's first MOTES owned circle the orb on ring i, each ring at its own speed. */
export const Motes = defineSystem("Motes", { reads: [Game], writes: [Mote, og.Transform, og.Sprite], after: ["Economy"] }, (q, time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  const n = counts(g);
  const show = g.phase !== Phase.Title;
  for (const a of q.archetypes()) {
    const m = a.column(Mote);
    const p = a.column(og.Transform).position;
    const sprite = a.column(og.Sprite);
    for (let i = 0; i < a.count; i++) {
      const tier = m.tier[i]!;
      const k = m.index[i]!;
      const visible = show && k < Math.min(MOTES, n[tier]!) ? 1 : 0;
      if (sprite.visible[i] !== visible) sprite.visible[i] = visible;
      if (!visible) continue;
      const ring = 1.75 + 0.55 * tier;
      const speed = (tier % 2 === 0 ? 1 : -1) * (0.5 - tier * 0.08);
      const angle = time.elapsed * speed + (k / MOTES) * Math.PI * 2 + tier * 0.7;
      p[i * 3] = g.orbX + Math.cos(angle) * ring;
      p[i * 3 + 1] = g.orbY + Math.sin(angle) * ring * 0.92;
      sprite.opacity[i] = 0.75 + 0.25 * Math.sin(time.elapsed * 3 + k);
    }
  }
});

/** Fires the burst the others asked for this tick from the pool of its kind (the oldest one). */
export const Bursts = defineSystem("Bursts", { reads: [Game], writes: [Burst, og.Transform, og.ParticleSystem], after: ["Motes"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.burstTick !== world.tick) return;
  let best = -1;
  let bestA: ReturnType<typeof q.archetypes>[number] | null = null;
  let oldest = Number.POSITIVE_INFINITY;
  for (const a of q.archetypes()) {
    const b = a.column(Burst);
    const fx = a.column(og.ParticleSystem);
    for (let i = 0; i < a.count; i++) {
      if (b.kind[i] !== g.burstKind) continue;
      const started = fx.visible[i] ? fx.startTick[i]! : -1;
      if (started < oldest) {
        oldest = started;
        best = i;
        bestA = a;
      }
    }
  }
  if (bestA === null || best < 0) return;
  const p = bestA.column(og.Transform).position;
  const fx = bestA.column(og.ParticleSystem);
  p[best * 3] = g.burstX;
  p[best * 3 + 1] = g.burstY;
  fx.visible[best] = 1;
  fx.startTick[best] = world.tick;
});

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next; the tap's pitch climbs with the streak. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Bursts"] }, (q, _time, world) => {
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
        if (b === Sound.Tap) src.pitch[i] = Math.pow(2, g.streak / 24) * (rng.ok ? rng.value.range(0.98, 1.02) : 1);
        if (b === Sound.Buy && rng.ok) src.pitch[i] = rng.value.range(0.95, 1.06);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
