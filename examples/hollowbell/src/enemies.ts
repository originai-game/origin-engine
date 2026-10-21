// The hollow's creatures, one system for all three kinds:
// - a beetle walks its beat (homeX ± range) and turns at the ends;
// - a moth hovers about its home on two sine waves and, seeing Wren within
//   Tuning.sight, dives at where Wren was (a 0.3 s telegraph first: it
//   folds its wings), then rises home;
// - the gloam, a beetle the size of a barrel, wakes when Wren enters its
//   hall, walks at Wren and every 2.5 s winds up (0.5 s, flashing red: the
//   telegraph, og-feel §2) and charges.
// Each takes the mallet's strike (Game.struck*) with five things at once:
// hit stop, a white flash and squash, a knock-back, camera shake and kick,
// a floating number (og-feel §3-§7). Each hurts Wren on contact. The
// gloam's fall is the heaviest moment of the run: a long hit stop, slow
// motion for 40 real ticks, a big shake, a burst, and the third chime.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Enemy, FxKind, Game, Kind, Phase, Sound, State, Tuning } from "./components.ts";
import { fx, hurtHero, pop, say } from "./feel.ts";
import { BEETLE_CLIP, MOTH_CLIP, must } from "./world.ts";

type GameV = Value<typeof Game>;

/** Glimmers each kind drops. */
const DROP = [2, 2, 8] as const;
/** The hurt state's length: no second strike lands in it. */
const HURT_TICKS = 16;
const ARENA = { min: 41.5, max: 58.5 } as const;

function struck(g: GameV, e: Entity): boolean {
  const id = e as unknown as number;
  return g.strike && (g.struck0 === id || g.struck1 === id || g.struck2 === id || g.struck3 === id);
}

/** The enemy's box against the hero's (feet at heroX, heroY; 0.7 x 1.25). */
function touches(g: GameV, kind: number, x: number, y: number): boolean {
  const [hw, y0, y1] = kind === Kind.Moth ? [0.4, y - 0.4, y + 0.4] : kind === Kind.Gloam ? [1.3, y, y + 1.6] : [0.6, y, y + 0.75];
  return Math.abs(g.heroX - x) < hw + 0.35 && g.heroY < y1 && g.heroY + 1.25 > y0;
}

export const Enemies = defineSystem(
  "Enemies",
  { reads: [Tuning], writes: [Game, Enemy, og.Transform, og.Flipbook, og.Sprite], after: ["Control"] },
  (q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const dt = time.scaledDt;
    const playing = g.phase === Phase.Play && !g.held;
    for (const a of q.archetypes()) {
      const en = a.column(Enemy);
      const tr = a.column(og.Transform);
      const fb = a.column(og.Flipbook);
      const sp = a.column(og.Sprite);
      for (let i = 0; i < a.count; i++) {
        const e = a.entity(i);
        const kind = en.kind[i]!;
        const bit = 1 << en.index[i]!;
        if (g.start) {
          // A new run or a continue: home, whole, or gone if the save says it was beaten.
          const dead = (g.killMask & bit) !== 0;
          tr.position[i * 3] = en.homeX[i]!;
          tr.position[i * 3 + 1] = en.homeY[i]!;
          tr.scale[i * 3] = tr.scale[i * 3 + 1] = 1;
          en.state[i] = dead ? State.Dead : State.Walk;
          en.timer[i] = dead ? 999 : 0;
          en.hp[i] = kind === Kind.Gloam ? t.gloamHp : kind === Kind.Beetle ? 2 : 1;
          en.knock[i] = 0;
          sp.visible[i] = dead ? 0 : 1;
          sp.opacity[i] = 1;
          sp.flash[i] = 0;
          if (kind === Kind.Gloam) {
            g.bossAwake = false;
            g.bossHp = en.hp[i]!;
          }
          continue;
        }
        if (!playing) continue;
        if (en.state[i] === State.Dead) {
          en.timer[i] = en.timer[i]! + 1;
          if (en.timer[i]! > 30) sp.visible[i] = 0;
          continue;
        }
        if (dt > 0) en.timer[i] = en.timer[i]! + 1;
        let x = tr.position[i * 3]!;
        let y = tr.position[i * 3 + 1]!;

        // The strike.
        // One strike lands once; the gloam shrugs off strikes for 18 ticks after one lands, and never reels.
        const fresh = kind === Kind.Gloam ? time.tick - en.hitAt[i]! > 18 || en.hitAt[i] === 0 : en.state[i] !== State.Hurt;
        if (struck(g, e) && en.hitAt[i] !== time.tick && fresh) {
          en.hitAt[i] = time.tick;
          en.hp[i] = en.hp[i]! - 1;
          g.hits += 1;
          g.sfx |= Sound.Hit;
          const crit = kind === Kind.Gloam && en.hp[i]! <= 0;
          og.time.hitStop(world, crit ? 14 : kind === Kind.Gloam ? t.hitStop + 2 : t.hitStop);
          og.tween.flash(world, { entity: e, duration: 0.14 });
          og.tween.squash(world, { entity: e, impact: { x: 1.3, y: 0.75 }, duration: 0.2 });
          og.camera.shake(world, kind === Kind.Gloam ? t.shake * 1.5 : t.shake);
          og.camera.kick(world, { x: g.facing, y: 0, z: 0 }, 0.12);
          pop(world, "-1", x, y + (kind === Kind.Gloam ? 2.4 : 1.2), "dealt", crit);
          fx(g, FxKind.Hit, g.strikeX + g.facing * 0.3, g.strikeY);
          en.knock[i] = g.facing * (kind === Kind.Gloam ? 1.5 : 7);
          if (kind === Kind.Gloam) g.bossHp = en.hp[i]!;
          if (en.hp[i]! <= 0) {
            en.state[i] = State.Dead;
            en.timer[i] = 0;
            g.kills += 1;
            g.killMask |= bit;
            g.glimmers += DROP[kind]!;
            g.sfx |= Sound.Glimmer;
            pop(world, `+${DROP[kind]}`, x, y + 2, "gold");
            og.tween.to(world, { entity: e, field: "scale", to: [1.5, 0, 1], duration: 0.4, ease: "backIn" });
            og.tween.to(world, { entity: e, component: og.Sprite, field: "opacity", to: [0], duration: 0.4, ease: "cubicIn" });
            if (kind === Kind.Gloam) {
              g.bossDown = true;
              g.bossX = x;
              g.bossY = y;
              // Slow motion for 40 real ticks: the Ui system owns the time scale and counts them down.
              g.slowTicks = 40;
              og.camera.shake(world, 0.6);
              og.camera.punch(world, 0.12);
              fx(g, FxKind.Burst, x, y + 1);
              g.sfx |= Sound.Boss;
              og.ui.toast(world, say(world, "toast.bossDown"), { tone: "success" });
            }
            continue;
          }
          if (kind !== Kind.Gloam) {
            en.state[i] = State.Hurt;
            en.timer[i] = 0;
          }
        }

        // Moving.
        const k = en.knock[i]!;
        if (k !== 0) {
          x += k * dt;
          en.knock[i] = Math.abs(k) < 0.2 ? 0 : k * 0.85;
        }
        const state = en.state[i]!;
        if (kind === Kind.Beetle) {
          if (state === State.Hurt) {
            if (en.timer[i]! >= HURT_TICKS) en.state[i] = State.Walk;
          } else {
            x += en.dir[i]! * t.beetleSpeed * dt;
            if (x > en.homeX[i]! + en.range[i]!) en.dir[i] = -1;
            if (x < en.homeX[i]! - en.range[i]!) en.dir[i] = 1;
          }
          x = Math.max(en.homeX[i]! - en.range[i]! - 0.5, Math.min(en.homeX[i]! + en.range[i]! + 0.5, x));
          setClip(fb, i, state === State.Hurt ? BEETLE_CLIP.hurt : BEETLE_CLIP.walk);
          sp.flipX[i] = en.dir[i]! < 0 ? 1 : 0;
        } else if (kind === Kind.Moth) {
          const near = Math.hypot(g.heroX - x, g.heroY + 0.7 - y) < t.sight;
          if (state === State.Walk) {
            // Hover about home.
            const s = g.time + en.index[i]! * 1.3;
            const hx = en.homeX[i]! + Math.sin(s * 1.1) * 1.6;
            const hy = en.homeY[i]! + Math.sin(s * 2.3) * 0.5;
            x += (hx - x) * Math.min(1, dt * 3);
            y += (hy - y) * Math.min(1, dt * 3);
            if (near && en.timer[i]! > 90 && g.respawn === 0) {
              en.state[i] = State.Wind;
              en.timer[i] = 0;
              en.tx[i] = g.heroX;
              en.ty[i] = g.heroY + 0.6;
            }
          } else if (state === State.Wind) {
            if (en.timer[i]! >= 18) {
              en.state[i] = State.Dive;
              en.timer[i] = 0;
            }
          } else if (state === State.Dive) {
            const dx = en.tx[i]! - x;
            const dy = en.ty[i]! - y;
            const d = Math.hypot(dx, dy);
            const step = t.mothSpeed * dt;
            if (d <= step || en.timer[i]! > 70) {
              en.state[i] = State.Rise;
              en.timer[i] = 0;
            } else {
              x += (dx / d) * step;
              y += (dy / d) * step;
            }
          } else if (state === State.Rise || state === State.Hurt) {
            x += (en.homeX[i]! - x) * Math.min(1, dt * 1.5);
            y += (en.homeY[i]! - y) * Math.min(1, dt * 1.5);
            if (en.timer[i]! > 50) {
              en.state[i] = State.Walk;
              en.timer[i] = 0;
            }
          }
          setClip(fb, i, state === State.Hurt ? MOTH_CLIP.hurt : state === State.Wind || state === State.Dive ? MOTH_CLIP.dive : MOTH_CLIP.fly);
          sp.flipX[i] = g.heroX < x ? 1 : 0;
        } else {
          // The gloam.
          if (!g.bossAwake) {
            if (g.arena) {
              g.bossAwake = true;
              en.timer[i] = 0;
              og.ui.toast(world, say(world, "toast.boss"), { tone: "danger" });
              og.camera.shake(world, 0.25);
            }
          } else if (state === State.Walk) {
            const dir = g.heroX < x ? -1 : 1;
            en.dir[i] = dir;
            x += dir * 1.5 * dt;
            if (en.timer[i]! > 110 && g.respawn === 0) {
              en.state[i] = State.Wind;
              en.timer[i] = 0;
              en.tx[i] = dir;
            }
          } else if (state === State.Wind) {
            // The telegraph: half a second, flashing red.
            sp.flash[i] = en.timer[i]! % 8 < 4 ? 0.6 : 0;
            if (en.timer[i]! >= 30) {
              en.state[i] = State.Dash;
              en.timer[i] = 0;
              sp.flash[i] = 0;
            }
          } else if (state === State.Dash) {
            x += en.tx[i]! * t.chargeSpeed * dt;
            if (en.timer[i]! >= 36 || x <= ARENA.min || x >= ARENA.max) {
              en.state[i] = State.Walk;
              en.timer[i] = 0;
              og.camera.shake(world, x <= ARENA.min || x >= ARENA.max ? 0.3 : 0);
            }
          } else if (state === State.Hurt) {
            if (en.timer[i]! >= HURT_TICKS) {
              en.state[i] = State.Walk;
            }
          }
          x = Math.max(ARENA.min, Math.min(ARENA.max, x));
          const now = en.state[i]!;
          setClip(fb, i, now === State.Hurt ? BEETLE_CLIP.hurt : now === State.Wind ? BEETLE_CLIP.wind : now === State.Dash ? BEETLE_CLIP.dash : BEETLE_CLIP.walk);
          sp.flipX[i] = en.dir[i]! < 0 ? 1 : 0;
          g.bossX = x;
          g.bossY = y;
        }
        tr.position[i * 3] = x;
        tr.position[i * 3 + 1] = y;

        // Contact hurts (not while it reels from a strike).
        if ((kind === Kind.Gloam ? g.bossAwake : en.state[i] !== State.Hurt) && touches(g, kind, x, y)) hurtHero(world, g, t, x);
      }
    }
  },
);

function setClip(fb: { clip: { [i: number]: number }; time: { [i: number]: number } }, i: number, clip: number): void {
  if (fb.clip[i] === clip) return;
  fb.clip[i] = clip;
  fb.time[i] = 0;
}

/** For commands: hurt the hero as a touch from x would (`g` is the command's own handle: FINDINGS.md #6). */
export function touchHero(world: World, g: GameV, fromX: number): void {
  hurtHero(world, g, must(world.resource(Tuning)), fromX);
}
