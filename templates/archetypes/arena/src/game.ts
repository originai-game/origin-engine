// Coin Duel's state and systems. Everything that moves the world reads
// og.PlayerInput, the per-player input lockstep makes the same on every
// peer; og.Input (this peer's keys) is read by one system only, Submit,
// which hands it to og.net.submit. That split is the whole of what makes the
// game play over a room (docs/design.md §2.10): a system that read og.Input
// to move a fighter would move it on this peer alone, and the other peer's
// og.Desync would name the field.

import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

/** The arena's floor, world units: (0, 0) to (WIDTH, HEIGHT). */
export const WIDTH = 16;
export const HEIGHT = 9;
/** A fighter's half size and the coin's: they touch when their boxes overlap. */
export const FIGHTER = 0.4;
export const COIN = 0.25;
/** og.PlayerInput.buttons: bit 0 dashes. */
export const DASH = 1;
/** Duel.sfx: the sounds asked for this tick. */
export const Sound = { Coin: 1, Dash: 2, Win: 4 } as const;
/** Gold, linear: the coin, its sparks. */
const GOLD_TEXT = { r: 1, g: 0.78, b: 0.25 };
/** Red and Blue for the "+1" over the taker. */
const TEXT_INKS = [
  { r: 1, g: 0.42, b: 0.36 },
  { r: 0.45, g: 0.68, b: 1 },
];

/** One of the two fighters: its own state beside the og.PlayerInput the engine writes. */
export const Fighter = defineComponent("Fighter", {
  /** Ticks left of a dash (moving at Tuning.dashSpeed), then of its cooldown below zero. */
  dash: f32(0, { unit: "ticks" }),
  /** The last move direction, for a dash without keys. */
  faceX: f32(1),
  faceY: f32(0),
});

/** The coin both fighters run for. */
export const Coin = defineComponent("Coin", {});

/** The round: the scores, the coin, the positions the coin and the HUD read. */
export const Duel = defineResource("Duel", {
  score0: u32(),
  score1: u32(),
  /** The fighter who reached Tuning.target first, plus one; 0 while playing. */
  winner: u32(),
  /** Ticks until the next round once a fighter won. */
  rest: u32(0, { unit: "ticks" }),
  /** The coin was taken this tick: CoinDrop moves it. */
  taken: u32(),
  coinX: f32(WIDTH / 2, { unit: "m" }),
  coinY: f32(HEIGHT / 2, { unit: "m" }),
  coins: u32(),
  /** Where each fighter stands, written by Move. */
  x0: f32(0, { unit: "m" }),
  y0: f32(0, { unit: "m" }),
  x1: f32(0, { unit: "m" }),
  y1: f32(0, { unit: "m" }),
  /** Sound.* bits asked for this tick; Feedback plays and clears them. */
  sfx: u32(),
  /** The coin taken this tick, by whom (player + 1), and where it lay: Feedback's sparks, "+1" and shake. */
  took: u32(),
  tookX: f32(0, { unit: "m" }),
  tookY: f32(0, { unit: "m" }),
});

/** A one-shot voice: Feedback starts it when its Sound bit is asked for. */
export const Voice = defineComponent("Voice", { bit: u32(), plays: u32() });
/** The coin's spark emitter: Feedback moves it where the coin was and restarts it. */
export const Sparks = defineComponent("Sparks", { bursts: u32() });

/** The tunables: `set:Tuning.<field>=<value>` and Tune write them; the same on every peer, or the worlds part. */
export const Tuning = defineResource("Tuning", {
  speed: f32(6, { unit: "m/s", min: 1, max: 20 }),
  dashSpeed: f32(16, { unit: "m/s", min: 1, max: 40 }),
  dashTicks: f32(8, { unit: "ticks", min: 1, max: 30 }),
  dashCooldown: f32(40, { unit: "ticks", min: 0, max: 240 }),
  target: u32(5, { min: 1, max: 99 }),
  restTicks: u32(120, { unit: "ticks", min: 1, max: 600 }),
});

export function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/**
 * A scripted player for playtests and the two-process run (`bot:<n>`):
 * this peer's input comes from the tick instead of the keys. It is input,
 * so it lives outside the world: each peer's bot drives only its own seat.
 */
let bot = 0;
export function setBot(n: number): void {
  if (!Number.isInteger(n) || n < 0) throw new Error("write bot:<n> with a whole number (0 hands the keys back)");
  bot = n;
}

/** The bot's input at a tick: towards the coin, weaving by its number, a dash every 90 ticks. */
function botInput(n: number, tick: number, from: { x: number; y: number }, to: { x: number; y: number }): { x: number; y: number; buttons: number } {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.max(Math.hypot(dx, dy), 1e-3);
  const weave = 0.7 * Math.sin(tick * 0.045 * (1 + 0.3 * n) + n);
  return { x: dx / len - (dy / len) * weave, y: dy / len + (dx / len) * weave, buttons: (tick + 23 * n) % 90 < 2 ? DASH : 0 };
}

/** This peer's keys (or pad, or bot) to og.net.submit: the only reader of og.Input. */
export const Submit = defineSystem("Submit", { reads: [og.Input, og.Gamepad, og.Lockstep, Duel] }, (_q, time, world) => {
  if (bot > 0) {
    const d = must(world.resource(Duel));
    const me = must(world.resource(og.Lockstep)).localPlayer === 1 ? { x: d.x1, y: d.y1 } : { x: d.x0, y: d.y0 };
    const input = botInput(bot, time.tick, me, { x: d.coinX, y: d.coinY });
    must(og.net.submit(world, { x: Math.max(-1, Math.min(1, input.x)), y: Math.max(-1, Math.min(1, input.y)), buttons: input.buttons }));
    return;
  }
  const input = must(world.resource(og.Input));
  const pad = must(world.resource(og.Gamepad));
  const sx = pad.axis("leftX", 0);
  const sy = pad.axis("leftY", 0);
  const x = Math.max(-1, Math.min(1, input.axis("KeyA", "KeyD") + input.axis("ArrowLeft", "ArrowRight") + (Math.abs(sx) > 0.25 ? sx : 0)));
  const y = Math.max(-1, Math.min(1, input.axis("KeyS", "KeyW") + input.axis("ArrowDown", "ArrowUp") - (Math.abs(sy) > 0.25 ? sy : 0)));
  const dash = input.down("Space") || input.down("ShiftLeft") || pad.down("a", 0);
  must(og.net.submit(world, { x, y, buttons: dash ? DASH : 0 }));
});

/** Each fighter from its player's og.PlayerInput: walk, dash, stay on the floor, take the coin. */
export const Move = defineSystem(
  "Move",
  { reads: [og.PlayerInput, Tuning], writes: [Fighter, og.Transform, og.Sprite, Duel], after: ["Submit"] },
  (q, time, world) => {
    const t = must(world.resource(Tuning));
    const duel = must(world.resource(Duel));
    const playing = duel.winner === 0;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const f = a.column(Fighter);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      for (let i = 0; i < a.count; i++) {
        let x = playing ? input.x[i]! : 0;
        let y = playing ? input.y[i]! : 0;
        const len = Math.hypot(x, y);
        if (len > 1) {
          x /= len;
          y /= len;
        }
        if (len > 0.1) {
          f.faceX[i] = x / Math.max(len, 1e-6);
          f.faceY[i] = y / Math.max(len, 1e-6);
        }
        if (playing && (input.buttons[i]! & DASH) !== 0 && f.dash[i]! <= -t.dashCooldown) {
          f.dash[i] = t.dashTicks;
          duel.sfx |= Sound.Dash;
        }
        let vx = x * t.speed;
        let vy = y * t.speed;
        if (f.dash[i]! > 0) {
          vx = f.faceX[i]! * t.dashSpeed;
          vy = f.faceY[i]! * t.dashSpeed;
        }
        if (f.dash[i]! > -t.dashCooldown) f.dash[i]! -= 1;
        p[i * 3] = Math.max(FIGHTER, Math.min(WIDTH - FIGHTER, p[i * 3]! + vx * time.dt));
        p[i * 3 + 1] = Math.max(FIGHTER, Math.min(HEIGHT - FIGHTER, p[i * 3 + 1]! + vy * time.dt));
        sprite.flash[i] = f.dash[i]! > 0 ? 0.6 : 0;
        if (input.player[i] === 0) {
          duel.x0 = p[i * 3]!;
          duel.y0 = p[i * 3 + 1]!;
        } else {
          duel.x1 = p[i * 3]!;
          duel.y1 = p[i * 3 + 1]!;
        }
        if (
          playing &&
          duel.taken === 0 &&
          Math.abs(p[i * 3]! - duel.coinX) < FIGHTER + COIN &&
          Math.abs(p[i * 3 + 1]! - duel.coinY) < FIGHTER + COIN
        ) {
          duel.taken = input.player[i]! + 1;
        }
      }
    }
  },
);

/** The coin taken: a point to its taker, the coin to a fresh place from og.Rng, and the round's end at Tuning.target. */
export const CoinDrop = defineSystem(
  "CoinDrop",
  { reads: [Coin, Tuning], writes: [og.Transform, Duel, og.Rng], after: ["Move"] },
  (q, _time, world) => {
    const t = must(world.resource(Tuning));
    const duel = must(world.resource(Duel));
    const rng = must(world.resource(og.Rng));
    if (duel.winner !== 0) {
      duel.rest -= 1;
      if (duel.rest === 0) {
        duel.winner = 0;
        duel.score0 = 0;
        duel.score1 = 0;
      }
    }
    if (duel.taken !== 0) {
      if (duel.taken === 1) duel.score0 += 1;
      else duel.score1 += 1;
      duel.coins += 1;
      duel.took = duel.taken;
      duel.tookX = duel.coinX;
      duel.tookY = duel.coinY;
      duel.sfx |= Sound.Coin;
      duel.taken = 0;
      duel.coinX = rng.range(1, WIDTH - 1);
      duel.coinY = rng.range(1, HEIGHT - 1);
      const best = Math.max(duel.score0, duel.score1);
      if (best >= t.target) {
        duel.winner = duel.score0 >= t.target ? 1 : 2;
        duel.rest = t.restTicks;
        duel.sfx |= Sound.Win;
      }
    }
    for (const a of q.archetypes()) {
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        p[i * 3] = duel.coinX;
        p[i * 3 + 1] = duel.coinY;
      }
    }
  },
);

/**
 * What a coin, a dash and a round won look and sound like (og-feel §3, §4, §7): the coin's
 * chime and a ring of sparks where it lay, "+1" in the taker's ink, a short hit stop and a
 * light shake; a dash's whoosh; the round's fanfare and a heavier shake. Everything is asked
 * for in Duel by the systems before and played here from the world alone, so every peer of a
 * room plays the same.
 */
export const Feedback = defineSystem(
  "Feedback",
  { writes: [Duel, Sparks, og.Transform, og.ParticleSystem], after: ["CoinDrop"] },
  (q, time, world) => {
    const duel = must(world.resource(Duel));
    if (duel.took === 0) return;
    const who = duel.took - 1;
    must(og.render.floatingText(world, { text: "+1", position: { x: duel.tookX, y: duel.tookY + 0.5, z: 1 }, color: TEXT_INKS[who] ?? GOLD_TEXT, size: 0.6, rise: 1.2, spread: 0.2, lifetime: 0.7 }));
    must(og.time.hitStop(world, 3));
    must(og.camera.shake(world, duel.winner !== 0 ? 0.45 : 0.15));
    for (const a of q.archetypes()) {
      const p = a.column(og.Transform).position;
      const ps = a.column(og.ParticleSystem);
      const sp = a.column(Sparks);
      for (let i = 0; i < a.count; i++) {
        p[i * 3] = duel.tookX;
        p[i * 3 + 1] = duel.tookY;
        ps.startTick[i] = time.tick;
        sp.bursts[i] = sp.bursts[i]! + 1;
      }
    }
    duel.took = 0;
  },
);

/** Plays each sound Duel.sfx asks for once; a voice still sounding stops this tick and starts on the next. */
export const Voices = defineSystem("Voices", { writes: [Duel, Voice, og.AudioSource], after: ["Feedback"] }, (q, _time, world) => {
  const duel = must(world.resource(Duel));
  if (duel.sfx === 0) return;
  let left = duel.sfx;
  for (const a of q.archetypes()) {
    const v = a.column(Voice);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = v.bit[i]!;
      if ((duel.sfx & b) === 0) continue;
      if (src.playing[i]) src.playing[i] = 0;
      else {
        src.playing[i] = 1;
        v.plays[i] = v.plays[i]! + 1;
        left &= ~b;
      }
    }
  }
  duel.sfx = left;
});

/** The HUD: both scores, and who this peer plays or whom it waits for. */
export const Hud = defineSystem("Hud", { reads: [Duel, og.Lockstep] }, (_q, _time, world) => {
  const duel = must(world.resource(Duel));
  const net = og.net.status(world);
  const lock = must(world.resource(og.Lockstep));
  og.ui.anchor("Score", { anchor: "top", direction: "row", gap: 32, padding: { left: 16, top: 12, right: 16, bottom: 12 } }, () => {
    og.ui.label("P1", `Red ${duel.score0}`, { step: "xl", color: { token: "danger" }, nowrap: true });
    og.ui.label("P2", `Blue ${duel.score1}`, { step: "xl", color: { token: "info" }, nowrap: true });
  });
  // This peer's own line: it differs between peers, so it is drawn and never kept in the world.
  const line =
    duel.winner !== 0
      ? `${duel.winner === 1 ? "Red" : "Blue"} wins`
      : net.running
        ? lock.localPlayer < lock.players
          ? `You are ${lock.localPlayer === 0 ? "Red" : "Blue"}`
          : "Watching"
        : net.peer === null
          ? "Local: Red"
          : `Waiting for ${Math.max(0, lock.players - net.peers.length)} more`;
  og.ui.anchor("Status", { anchor: "bottom", padding: { left: 16, top: 12, right: 16, bottom: 16 } }, () => {
    og.ui.label("Line", line, { color: { token: "inkMuted" }, nowrap: true });
  });
});

/** The facts `og run --json` and the probe print. */
export function facts(world: World): Record<string, unknown> {
  const d = world.resource(Duel);
  if (!d.ok) return {};
  return { score0: d.value.score0, score1: d.value.score1, coins: d.value.coins, winner: d.value.winner };
}
