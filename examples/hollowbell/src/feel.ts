// The juice every system shares: effect requests, the hero's hurt, the
// strings. Amounts follow og-feel (§3 hit stop, §4 trauma, §5 squash, §7
// floating text).

import { og } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { FxKind, Game, HEARTS, Sound, Tuning } from "./components.ts";

type GameV = Value<typeof Game>;
type TuningV = Value<typeof Tuning>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

/** Ask the Fx system for a burst of `kind` at x, y this tick. */
export function fx(g: GameV, kind: number, x: number, y: number): void {
  g.fx |= kind;
  if (kind === FxKind.Dust) {
    g.dustX = x;
    g.dustY = y;
  } else if (kind === FxKind.Hit) {
    g.hitX = x;
    g.hitY = y;
  } else if (kind === FxKind.Chime) {
    g.chimeX = x;
    g.chimeY = y;
  } else {
    g.burstX = x;
    g.burstY = y;
  }
}

const RED = { r: 0.93, g: 0.3, b: 0.25 };
const GOLD = { r: 1, g: 0.85, b: 0.35 };

/** A floating number in the meaning's colour (og-feel §7): white dealt, red taken, gold glimmers. */
export function pop(world: World, text: string, x: number, y: number, tone: "dealt" | "taken" | "gold", crit = false): void {
  const color = tone === "taken" ? RED : tone === "gold" ? GOLD : { r: 1, g: 1, b: 1 };
  og.render.floatingText(world, { text, position: { x, y, z: 0 }, color, size: 0.55, rise: 1.1, spread: 0.3, lifetime: 0.8, crit });
}

/** Wren is hurt by something at `fromX`: a heart, the knock-back, red flash, hit stop, shake, the sound, "-1". */
export function hurtHero(world: World, g: GameV, t: TuningV, fromX: number): void {
  if (g.invulnerable > 0 || g.respawn > 0 || g.hearts <= 0) return;
  const hero = g.hero as unknown as Entity;
  g.hearts -= 1;
  g.hurts += 1;
  g.invulnerable = t.iframes;
  g.knock = t.knockTicks;
  g.knockDir = g.heroX >= fromX ? 1 : -1;
  g.sfx |= Sound.Hurt;
  og.tween.flash(world, { entity: hero, color: RED, duration: 0.15 });
  og.tween.squash(world, { entity: hero, impact: { x: 1.2, y: 0.8 }, duration: 0.15 });
  og.time.hitStop(world, 4);
  og.camera.shake(world, 0.32);
  og.camera.kick(world, { x: g.knockDir, y: 0, z: 0 }, 0.18);
  pop(world, "-1", g.heroX, g.heroY + 1.6, "taken");
  if (g.hearts <= 0) {
    // Every heart gone: Wren falls, and wakes by the last lantern.
    g.falls += 1;
    g.respawn = 90;
    og.time.hitStop(world, 14);
    og.camera.shake(world, 0.55);
  }
}

/** Hearts back to full (a lantern, a respawn). */
export function heal(g: GameV): void {
  g.hearts = HEARTS;
}
