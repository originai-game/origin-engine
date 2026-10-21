// Wick: the keyboard and the pad into og.PlatformerController (run, jump with
// coyote time and a buffer, wall slide and wall jump, grates dropped
// through with Down), the push (the controller shoves a crate it runs into;
// Wick slows to Tuning.pushSpeed while a crate is in front of him), the
// flipbook clip by what he is doing, the hurt and the respawn at the last
// lamp, warps, the stomp's bounce, and the feel of each (sounds, dust,
// squash and stretch, shake). Per-entity calls on the one named entity
// (og-core §4: the player).

import { og, defineSystem, CHARACTER_JUMPED, CHARACTER_LANDED, CHARACTER_WALL_JUMPED, SHAPE_BOX } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Bits, Fx, FxKind, Game, Hero, Phase, Sfx, Sound, Tuning } from "./components.ts";
import { effect } from "./feel.ts";
import { must } from "./level.ts";

type GameV = Value<typeof Game>;

/** The clips of sprites/wick.ogt, by index (og.anim.clip resolves them in order). */
export const Clip = { idle: 0, run: 1, jump: 2, fall: 3, wall: 4, push: 5, hurt: 6, cheer: 7 } as const;

/** What the player holds this tick: keys (arrows, WASD, Space, Z) and pad 0 (stick, d-pad, A; Y retries). */
export function intent(world: World, g: GameV): { x: number; jump: boolean; down: boolean; retry: boolean } {
  const k = must(world.resource(og.Input));
  const pad = must(world.resource(og.Gamepad));
  let x = (k.down("ArrowRight") || k.down("KeyD") ? 1 : 0) - (k.down("ArrowLeft") || k.down("KeyA") ? 1 : 0);
  if (pad.connected(0)) {
    const stick = pad.axis("leftX", 0);
    if (Math.abs(stick) > 0.3) x = Math.sign(stick);
    if (pad.down("right", 0)) x = 1;
    if (pad.down("left", 0)) x = -1;
  }
  if (Math.abs(g.touchX) > 0.3) x = Math.sign(g.touchX);
  const jump = g.touchJump || k.down("Space") || k.down("KeyZ") || k.down("ArrowUp") || k.down("KeyW") || (pad.connected(0) && pad.down("a", 0));
  const down = k.down("ArrowDown") || k.down("KeyS") || (pad.connected(0) && (pad.down("down", 0) || pad.axis("leftY", 0) > 0.6));
  const retry = k.pressed("KeyR") || (pad.connected(0) && pad.pressed("y", 0));
  return { x, jump, down, retry };
}

/** Puts Wick's feet at (x, y) at once and cancels his velocity (warps, respawns, a new level). */
function place(world: World, wick: Entity, x: number, y: number): void {
  must(world.set(wick, og.Transform, { position: { x, y, z: 0 } }));
  const c = must(world.get(wick, og.PlatformerController));
  // The controller's velocity is written by the engine; an impulse cancels it on the next step.
  must(world.set(wick, og.PlatformerController, { impulse: { x: -c.velocity.x, y: -c.velocity.y }, moveX: 0, jump: false }));
}

/** Something hurt Wick: Control answers on the next tick (it owns him). */
export function hurtWick(g: GameV): void {
  if (g.hurt > 0 || g.safe > 0 || g.exiting > 0 || g.phase !== Phase.Play) return;
  g.hurtAsk = true;
}

/** The hurt itself: Wick drops his flame and is knocked back, the camera shakes, the respawn follows. */
function takeHurt(world: World, wick: Entity, g: GameV, t: Value<typeof Tuning>, sfx: Value<typeof Sfx>): void {
  g.hurtAsk = false;
  g.hurt = t.hurtTicks;
  g.deaths += 1;
  sfx.bits |= Sound.Hurt;
  og.camera.shake(world, 0.35);
  og.time.hitStop(world, 6);
  og.tween.flash(world, { entity: wick, amount: 1, duration: 0.15 });
  const v = must(world.get(wick, og.PlatformerController)).velocity;
  must(world.set(wick, og.PlatformerController, { impulse: { x: -v.x - g.facing * 4, y: 6 - v.y } }));
}

/** A crate just in front of Wick's knees, the way he faces. */
function crateAhead(world: World, wick: Entity, x: number, y: number, facing: number): boolean {
  const hit = og.physics.overlap(world, {
    shape: { shape: SHAPE_BOX, size: { x: 0.3, y: 0.6, z: 1 } },
    position: { x: x + facing * 0.55, y: y + 0.45, z: 0 },
    filter: { mask: Bits.Crate, exclude: wick },
  });
  return hit.ok && hit.value.length > 0;
}

export const Control = defineSystem(
  "Control",
  {
    reads: [og.Input, og.Gamepad, Tuning, og.CharacterEvent, og.FlipbookEvent],
    writes: [Game, Sfx, Fx, Hero, og.PlatformerController, og.Transform, og.Sprite, og.Flipbook],
  },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const sfx = must(world.resource(Sfx));
    const fx = must(world.resource(Fx));
    const wick = world.find("wick");
    if (wick === null) return;

    if (g.warp) {
      g.warp = false;
      place(world, wick, g.warpX, g.warpY);
    }
    const pos = must(world.get(wick, og.Transform)).position;
    const c = must(world.get(wick, og.PlatformerController));
    g.heroX = pos.x;
    g.heroY = pos.y;
    g.heroVY = c.velocity.y;
    g.grounded = c.grounded;
    g.onWall = c.onWall;

    const playing = g.phase === Phase.Play && !g.paused && !g.settingsOpen && g.exiting === 0 && g.hurt === 0;
    const want = playing ? intent(world, g) : { x: 0, jump: false, down: false, retry: false };

    // The last lamp, on R or the pad's Y: a fall into a corner without a way out is never the end.
    if (playing && want.retry) {
      g.hurt = 1;
      g.deaths += 1;
    }
    if (g.hurtAsk) {
      if (g.hurt === 0 && g.safe === 0 && g.exiting === 0 && g.phase === Phase.Play) takeHurt(world, wick, g, t, sfx);
      else g.hurtAsk = false;
    } else if (g.hurt > 0) {
      g.hurt -= 1;
      if (g.hurt === 0) {
        // Back at the last lamp (or the spawn), the level's crates where that lamp saw them.
        g.warp = false;
        place(world, wick, g.checkX, g.checkY);
        g.safe = t.safeTicks;
        g.resetLevel = g.level;
        g.reset = 1;
        g.heroX = g.checkX;
        g.heroY = g.checkY;
      }
    } else if (g.safe > 0) g.safe -= 1;

    // A stomp bounces him off the tick (og-feel §3: an impulse, taken on the next step).
    if (g.bounce) {
      g.bounce = false;
      must(world.set(wick, og.PlatformerController, { impulse: { x: 0, y: t.bounce - c.velocity.y } }));
      og.tween.squash(world, { entity: wick, impact: { x: 0.85, y: 1.15 }, duration: 0.12 });
    }

    // Falling out of the vault counts as a hurt.
    if (g.phase === Phase.Play && g.heroY < -40) hurtWick(g);

    const facing = want.x !== 0 ? want.x : c.facing;
    const pushing = playing && c.grounded && want.x !== 0 && crateAhead(world, wick, pos.x, pos.y, facing);
    g.pushing = pushing;
    must(world.set(wick, og.PlatformerController, {
      moveX: want.x,
      jump: want.jump,
      down: want.down,
      maxSpeed: pushing ? t.pushSpeed : t.maxSpeed,
    }));

    for (const e of world.events(og.CharacterEvent)) {
      if (e.entity !== wick) continue;
      if (e.kind === CHARACTER_JUMPED || e.kind === CHARACTER_WALL_JUMPED) {
        sfx.bits |= Sound.Jump;
        if (e.kind === CHARACTER_WALL_JUMPED) {
          g.wallJumps += 1;
          effect(fx, FxKind.Dust, pos.x - g.facing * 0.4, pos.y + 0.6, time.tick);
        } else g.jumps += 1;
        og.tween.squash(world, { entity: wick, impact: { x: 0.85, y: 1.15 }, duration: 0.1 });
      } else if (e.kind === CHARACTER_LANDED) {
        g.landings += 1;
        sfx.bits |= Sound.Land;
        const hard = e.speed < -12;
        effect(fx, FxKind.Dust, pos.x, pos.y, time.tick);
        og.tween.squash(world, { entity: wick, impact: hard ? { x: 1.3, y: 0.7 } : { x: 1.15, y: 0.85 }, duration: hard ? 0.18 : 0.12 });
        if (hard) og.camera.shake(world, 0.12);
      }
    }
    for (const e of world.events(og.FlipbookEvent)) {
      if (e.entity !== wick) continue;
      if (e.name === "step") {
        g.steps += 1;
        sfx.bits |= Sound.Step;
      } else if (e.name === "shove") {
        g.shoves += 1;
        sfx.bits |= Sound.Shove;
      }
    }

    // The clip by what he does; on a wall he faces it.
    const vx = c.velocity.x;
    const vy = c.velocity.y;
    let clip: number = Clip.idle;
    let speed = 1;
    if (g.hurt > 0) clip = Clip.hurt;
    else if (g.exiting > 0) clip = Clip.cheer;
    else if (!c.grounded) clip = c.onWall !== 0 && vy < 0 ? Clip.wall : vy > 0 ? Clip.jump : Clip.fall;
    else if (pushing) clip = Clip.push;
    else if (Math.abs(vx) > 0.4) {
      clip = Clip.run;
      // The feet must not skate (og-feel §9).
      speed = Math.max(0.5, Math.abs(vx) / t.maxSpeed);
    }
    const face = clip === Clip.wall ? c.onWall : facing;
    if (face !== 0) g.facing = face;
    const hero = must(world.get(wick, Hero));
    const fb = must(world.get(wick, og.Flipbook));
    if (hero.anim !== clip || fb.clip !== clip) {
      must(world.set(wick, og.Flipbook, { clip, time: 0, speed }));
      must(world.set(wick, Hero, { anim: clip }));
    } else if (fb.speed !== speed) must(world.set(wick, og.Flipbook, { speed }));
    must(world.set(wick, og.Sprite, { flipX: g.facing < 0, opacity: g.safe > 0 && Math.floor(g.safe / 4) % 2 === 1 ? 0.45 : 1 }));
  },
);
