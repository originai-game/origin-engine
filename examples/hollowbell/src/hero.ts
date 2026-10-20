// Wren on og.PlatformerController: the arrows (or A/D, the pad's stick and
// d-pad) run, Space (Z, the pad's A) jumps (held for the full height,
// released early for a hop), Down drops through a plank, X (J, the pad's X)
// swings the mallet, Up (W, the pad's Y) talks to the mole or rings the
// bell. The input is read and answered on the same tick (og-feel §1); the
// controller has the coyote time, the jump buffer, the variable height and
// the one-way planks, and the moth-wing cloak gives it one air jump.
//
// The swing's strike lands on the attack clip's `hit` frame: an
// og.physics.overlap of the mallet's box on the enemy and stone layers,
// written into Game for the Enemies and Stones systems. Footsteps come from
// the run clip's `step` events, jumps and landings from og.CharacterEvent
// (a stretch, a squash, dust, a sound).

import { og, defineSystem, CHARACTER_AIR_JUMPED, CHARACTER_JUMPED, CHARACTER_LANDED, SHAPE_BOX } from "@origingame/engine";
import type { Entity } from "@origingame/engine";
import { Bits, FxKind, Game, Hero, Phase, Sound, Tuning } from "./components.ts";
import { fx, heal, say } from "./feel.ts";
import { HERO_CLIP, must } from "./world.ts";

const DEAD_ZONE = 0.25;

export const Control = defineSystem(
  "Control",
  {
    reads: [Tuning, og.Input, og.Gamepad, og.FlipbookEvent, og.CharacterEvent],
    writes: [Game, Hero, og.Transform, og.PlatformerController, og.Flipbook, og.Sprite, og.Light2D],
  },
  (q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const input = must(world.resource(og.Input));
    const pad = must(world.resource(og.Gamepad));
    const playing = g.phase === Phase.Play && !g.held;
    const heroEntity = g.hero as unknown as Entity;

    // Frame events of this tick: the swing's strike and the footsteps.
    let strikeNow = false;
    for (const e of world.events(og.FlipbookEvent)) {
      if ((e.entity as unknown as number) !== g.hero) continue;
      if (e.name === "hit") strikeNow = true;
      if (e.name === "step" && g.grounded) {
        g.sfx |= Sound.Step;
        g.steps += 1;
      }
    }
    let jumped = false;
    let landed = 0;
    for (const e of world.events(og.CharacterEvent)) {
      if ((e.entity as unknown as number) !== g.hero) continue;
      if (e.kind === CHARACTER_JUMPED || e.kind === CHARACTER_AIR_JUMPED) jumped = true;
      if (e.kind === CHARACTER_LANDED) landed = Math.abs(e.speed);
    }

    g.strike = false;
    g.struck0 = g.struck1 = g.struck2 = g.struck3 = 0;

    for (const a of q.archetypes()) {
      const tr = a.column(og.Transform);
      const pc = a.column(og.PlatformerController);
      const fb = a.column(og.Flipbook);
      const sp = a.column(og.Sprite);
      const hero = a.column(Hero);
      const light = a.column(og.Light2D);
      for (let i = 0; i < a.count; i++) {
        const x = tr.position[i * 3]!;
        const y = tr.position[i * 3 + 1]!;

        // A warp (a new run, a continue, a respawn, the pit, a command) lands first.
        if (g.warp) {
          tr.position[i * 3] = g.warpX;
          tr.position[i * 3 + 1] = g.warpY;
          g.warp = false;
        }
        if (g.start) {
          hero.attack[i] = 0;
          hero.airJumped[i] = 0;
        }

        // The run's own clocks.
        if (playing) {
          if (g.invulnerable > 0) g.invulnerable -= 1;
          if (g.knock > 0 && time.scaledDt > 0) g.knock -= 1;
          if (g.respawn > 0 && time.scaledDt > 0) {
            g.respawn -= 1;
            if (g.respawn === 0) {
              // Wake by the last lantern (or the spawn) with every heart.
              heal(g);
              g.warp = true;
              g.warpX = g.checkpoint >= 0 ? g.checkX : g.spawnX;
              g.warpY = g.checkpoint >= 0 ? g.checkY : g.spawnY;
              g.invulnerable = t.iframes;
              og.ui.toast(world, say(world, "toast.woke"));
            }
          }
        }

        // Input, read and answered this tick.
        const stick = pad.axis("leftX", 0);
        let move = input.axis("ArrowLeft", "ArrowRight") + input.axis("KeyA", "KeyD") + (Math.abs(stick) > DEAD_ZONE ? stick : 0) + (pad.down("right", 0) ? 1 : 0) - (pad.down("left", 0) ? 1 : 0);
        move = Math.max(-1, Math.min(1, move));
        const jump = input.down("Space") || input.down("KeyZ") || pad.down("a", 0);
        const down = input.down("ArrowDown") || input.down("KeyS") || pad.down("down", 0) || pad.axis("leftY", 0) > 0.6;
        const swing = input.pressed("KeyX") || input.pressed("KeyJ") || pad.pressed("x", 0);
        const up = input.pressed("ArrowUp") || input.pressed("KeyW") || pad.pressed("y", 0) || pad.pressed("up", 0);

        const control = playing && g.respawn === 0 && g.knock === 0 && !g.talking && !g.shopping && g.ringing === 0;
        pc.maxSpeed[i] = t.maxSpeed;
        pc.jumpHeight[i] = t.jumpHeight;
        pc.timeToApex[i] = t.timeToApex;
        pc.airJumps[i] = g.cloak ? 1 : 0;
        pc.moveX[i] = control ? move : g.knock > 0 && playing ? g.knockDir : 0;
        pc.jump[i] = control && jump ? 1 : 0;
        pc.down[i] = control && down ? 1 : 0;

        if (control && move !== 0 && hero.attack[i] === 0) g.facing = move > 0 ? 1 : -1;
        if (control && swing && hero.attack[i] === 0) {
          hero.attack[i] = 1;
          g.swings += 1;
          g.sfx |= Sound.Swing;
        }
        if (control && up) {
          if (g.nearMole) {
            g.talking = true;
            g.talkIndex = 0;
            g.talks += 1;
          } else if (g.nearBell) {
            if (g.chimes >= 3) {
              g.ringing = 1;
            } else {
              og.ui.toast(world, say(world, "hud.need", { n: 3 - g.chimes }), { tone: "warn" });
            }
          }
        }

        // The strike: the mallet's box one reach ahead of the chest, on the enemy and stone layers.
        if (strikeNow && playing) {
          const sx = x + g.facing * t.reach * 0.7;
          const sy = y + 0.75;
          const hits = og.physics.overlap(world, {
            shape: { shape: SHAPE_BOX, size: { x: t.reach * 1.3, y: 1.3, z: 1 } },
            position: { x: sx, y: sy, z: 0 },
            filter: { mask: Bits.Enemy | Bits.Stone, includeSensors: true, exclude: heroEntity },
          });
          g.strike = true;
          g.strikeX = sx;
          g.strikeY = sy;
          if (hits.ok) {
            const list = hits.value.map((e) => e as unknown as number);
            g.struck0 = list[0] ?? 0;
            g.struck1 = list[1] ?? 0;
            g.struck2 = list[2] ?? 0;
            g.struck3 = list[3] ?? 0;
          }
        }

        // Jumps and landings: a stretch, a squash, dust, a sound, on the tick the controller reports them.
        if (jumped) {
          g.jumps += 1;
          g.sfx |= Sound.Jump;
          og.tween.squash(world, { entity: heroEntity, impact: { x: 0.85, y: 1.15 }, duration: 0.1 });
          fx(g, FxKind.Dust, x, y);
        }
        if (landed > 0 && g.respawn === 0) {
          g.landings += 1;
          g.sfx |= Sound.Land;
          const big = landed > 9;
          og.tween.squash(world, { entity: heroEntity, impact: big ? { x: 1.3, y: 0.7 } : { x: 1.15, y: 0.85 }, duration: big ? 0.18 : 0.12 });
          fx(g, FxKind.Dust, x, y);
        }

        // The clip: hurt, the swing, in the air, running, standing.
        const vx = pc.velocity[i * 3]!;
        const vy = pc.velocity[i * 3 + 1]!;
        const grounded = pc.grounded[i] === 1;
        let clip: number = HERO_CLIP.idle;
        if (hero.attack[i]! > 0) {
          hero.attack[i] = hero.attack[i]! + 1;
          if (fb.clip[i] === HERO_CLIP.attack && fb.finished[i] === 1 && hero.attack[i]! > 3) hero.attack[i] = 0;
        }
        if (g.respawn > 0 || g.knock > 0) clip = HERO_CLIP.hurt;
        else if (hero.attack[i]! > 0) clip = HERO_CLIP.attack;
        else if (!grounded) clip = vy > 0 ? HERO_CLIP.jump : HERO_CLIP.fall;
        else if (Math.abs(vx) > 0.4) clip = HERO_CLIP.run;
        if (fb.clip[i] !== clip) {
          fb.clip[i] = clip;
          fb.time[i] = 0;
        }
        // Feet that do not skate: the run plays at the speed it moves (og-feel §9).
        fb.speed[i] = clip === HERO_CLIP.run ? Math.max(0.4, Math.abs(vx) / t.maxSpeed) : 1;
        sp.flipX[i] = g.facing < 0 ? 1 : 0;
        // Blinking while invulnerable.
        sp.opacity[i] = g.invulnerable > 0 && g.respawn === 0 && Math.floor(g.invulnerable / 4) % 2 === 1 ? 0.35 : 1;

        // The lantern Wren carries: the brighter wick reaches farther.
        const want = g.wick ? t.wickRadius : t.lightRadius;
        light.radius[i] = light.radius[i]! + (want - light.radius[i]!) * 0.05;
        g.light = light.radius[i]!;

        hero.wasGrounded[i] = grounded ? 1 : 0;
        g.heroX = tr.position[i * 3]!;
        g.heroY = tr.position[i * 3 + 1]!;
        g.grounded = grounded;
      }
    }
  },
);
