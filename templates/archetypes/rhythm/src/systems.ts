// Three-Lane Beat's systems, in tick order after Seats (src/seats.ts):
// Controls (each seat's presses from its og.PlayerInput, the menu's and the
// pause's acted on), the screens (src/ui.ts), Clock (the song's og.Music
// clock follows Game.songStart), Play (presses,
// judgements, misses, the notes' scheduling by og.MusicBeat, drawing the
// highway), Band (the song's stems started on their bar lines by
// og.MusicBeat, the count-in's clicks), Lanes (the receptors' flash,
// squash and burst, the lanes' glow on the beat), Glows and Voices.
//
// Time is the world's: tick / tick rate, the clock og.music and og.audio
// keep, so a stem started at `songStart + bar × barLength` sounds on its
// bar line to the sample. A press is judged on the tick its bit goes down
// in og.PlayerInput, at the time it happened (the tick it was read and its
// event's `at` within it, aimX) less its device's lag (the output's latency
// and the calibrated offset, aimY: src/device.ts) against its note's exact
// time, so every peer of a room judges it the same.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Glow, Judgement, JUDGEMENTS, LANES, Note, Phase, Prefs, Receptor, Rival, Sfx, Sound, Stem, Tuning } from "./components.ts";
import { bit, ONLINE, SEATS } from "./seats.ts";
import { begin, calibrate, pause, resume, say, toTitle } from "./ui.ts";

type GameV = Value<typeof Game>;
/** A seat's presses and tally: Game's seat fields or Rival. */
type SeatV = Value<typeof Rival>;
type TuneV = Value<typeof Tuning>;

/** The song's stems, in the order `start` spawns them. */
export const PARTS = ["drums", "bass0", "bass1", "bass2", "bass3", "chords0", "chords1", "chords2", "chords3", "leadA", "leadB", "click0", "click1", "click2", "click3"] as const;

/** Each lane's colour: cyan, magenta, gold. */
export const LANE_COLOURS = [
  { r: 0.22, g: 0.88, b: 1 },
  { r: 1, g: 0.3, b: 0.85 },
  { r: 1, g: 0.82, b: 0.25 },
] as const;
const JUDGE_COLOURS = [{ r: 1, g: 0.9, b: 0.35 }, { r: 0.4, g: 0.95, b: 1 }, { r: 0.85, g: 0.9, b: 1 }, { r: 1, g: 0.35, b: 0.4 }] as const;

/** The camera feel's scale: 0.3 under reduced motion (og-feel), 1 otherwise. */
export function feel(world: World): number {
  const p = world.resource(Prefs);
  return p.ok && p.value.reducedMotion ? 0.3 : 1;
}

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

/** The world's time in seconds: the tick over the tick rate (og.music's and og.audio's clock). */
export function worldTime(world: World): number {
  return world.tick / 60;
}

/** The song's beat length in seconds (Game.beatLength, set from the chart's Song). */
export function beatLength(g: GameV): number {
  return 60 / g.bpm;
}

/** The stems that start on a bar (from 1): drums every bar; the bass, chords and lead on odd bars, two bars long. */
export function partsAt(bar: number, bars: number): string[] {
  const out: string[] = [];
  if (bar < 1 || bar > bars) return out;
  out.push("drums");
  if (bar % 2 === 1) {
    // The two-bar chords Am, F, C, G in turn from bar 5.
    const chord = (((bar - 5) / 2) % 4 + 4) % 4;
    if (bar >= 5 && bar <= 21) out.push(`bass${chord}`);
    if (bar >= 5 && bar <= 23) out.push(`chords${chord}`);
    if (bar >= 9 && bar <= 19) out.push(((bar - 9) / 2) % 2 === 0 ? "leadA" : "leadB");
  }
  return out;
}

/** The song's position in beats at a world time (negative in the count-in). */
export function beatAt(g: GameV, t: number): number {
  return (t - g.songStart) / beatLength(g);
}

/** Where the play is in world time: frozen while paused. */
function songNow(world: World, g: GameV): number {
  return g.pausedAt >= 0 ? g.pausedAt : worldTime(world);
}

/** The seats' state: Game carries seat 0's, Rival seat 1's. */
export function seatPlays(world: World): SeatV[] {
  const out: SeatV[] = [must2(world.resource(Game))];
  if (SEATS > 1) out.push(must2(world.resource(Rival)));
  return out;
}

/** Where a seat's highway is centred (seat 0 left, seat 1 right of the middle; the middle alone). */
export function highwayX(g: GameV, seat: number): number {
  return SEATS === 1 ? 0 : (seat === 0 ? -1 : 1) * g.spread;
}

/** The lanes' bits of og.PlayerInput.buttons (src/creation.ts's lane0 to lane2). */
const LANE_BITS = Array.from({ length: LANES }, (_, lane) => bit(`lane${lane}`));

/**
 * Each seat's presses from its og.PlayerInput: the controls that went
 * down this tick, when they happened (aimX) and its device's lag (aimY);
 * then the menu's and the pause's: a start (Play, Restart, Play again),
 * back to the title, the calibration, pause and resume, each as of the
 * tick it was pressed on, for every seat at once.
 */
export const Controls = defineSystem("Controls", { reads: [og.PlayerInput, Tuning], writes: [Game, Rival] }, (q, _time, world) => {
  const g = must2(world.resource(Game));
  const plays = seatPlays(world);
  let acts = 0;
  let when = Infinity;
  for (const a of q.archetypes()) {
    const inp = a.column(og.PlayerInput);
    for (let i = 0; i < a.count; i++) {
      const s = plays[inp.player[i]!];
      if (s === undefined) continue;
      const buttons = inp.buttons[i]!;
      const pressed = buttons & ~s.before;
      if (s.before !== buttons) s.before = buttons;
      if (s.pressed !== pressed) s.pressed = pressed;
      if (s.lagMs !== inp.aimY[i]) s.lagMs = inp.aimY[i]!;
      if (pressed === 0) continue;
      s.pressAt = (world.tick + inp.aimX[i]!) / 60;
      acts |= pressed;
      when = Math.min(when, Math.floor(world.tick + inp.aimX[i]!) / 60);
    }
  }
  if (acts === 0) return;
  // The menu's presses act as of the tick they were made on.
  const now = when;
  if (acts & bit("start") && (g.phase !== Phase.Play || g.paused)) begin(world, g, 1, now);
  else if (acts & bit("title")) toTitle(g);
  else if (acts & bit("calibrate") && !ONLINE) {
    if (g.phase === Phase.Calibrate) {
      g.calTaps = 0;
      g.calSum = 0;
    } else if (g.phase === Phase.Title) calibrate(world, g, now);
  } else if (acts & bit("resume")) resume(world, g, now);
  else if (acts & bit("pause")) pause(g, now);
});

/**
 * The play. Each tick: the presses (the count-in's taps and a calibration's
 * are measured, not judged), each pressed lane's nearest unjudged note of
 * its seat within the Good window judged by its error, unhit notes past the
 * window missed, the notes og.MusicBeat brings onto the highways, the
 * highways drawn, and the song's end (cleared a beat after its last note,
 * failed when the groove runs out: in versus when every seat's has).
 */
export const Play = defineSystem(
  "Play",
  { reads: [og.Camera, og.MusicBeat, Tuning, Prefs], writes: [Game, Rival, Note], after: ["Clock"] },
  (q, _time, world) => {
    const game = world.resource(Game);
    if (!game.ok) return;
    const g = game.value;
    const plays = seatPlays(world);
    const t = lazyTuning(world);
    const now = worldTime(world);
    const live = g.phase === Phase.Play && !g.held && g.pausedAt < 0;
    const calibrating = g.phase === Phase.Calibrate;
    // Each seat's lanes pressed this tick, and when each press was heard: its time less its device's lag.
    let mask = 0;
    plays.forEach((s, seat) => {
      if (!live && !calibrating) return;
      LANE_BITS.forEach((b, lane) => {
        if (s.pressed & b) mask |= 1 << (seat * LANES + lane);
      });
    });
    const heard = plays.map((s) => s.pressAt - s.lagMs / 1000);
    if (mask !== g.pressMask) g.pressMask = mask;
    plays.forEach((s, seat) => {
      const lanes = (mask >> (seat * LANES)) & 7;
      if (lanes === 0) return;
      s.taps += 1;
      for (let lane = 0; lane < LANES; lane++) if (lanes & (1 << lane)) g.sfx |= Sound.Tap0 << lane;
      // Calibration: each tap's error from the nearest click (every beat of the clock).
      if (calibrating && g.calTaps < 8 && now > g.songStart) {
        const len = beatLength(g);
        const b = (heard[seat]! - g.songStart) / len;
        g.calTaps += 1;
        g.calSum += (b - Math.round(b)) * len * 1000;
      }
    });
    // The notes og.MusicBeat brings: every note up to scrollBeats past the beat just sent, and one more
    // (a start or a resume asks for the first ones through Game.scheduleUpTo).
    let upTo = g.scheduleUpTo;
    if (g.phase === Phase.Play && g.pausedAt < 0) {
      for (const e of world.events(og.MusicBeat)) upTo = Math.max(upTo, (e.bar - 1) * 4 + (e.beat - 1) + 1 + t.scrollBeats);
    }
    if (g.scheduleUpTo !== -1) g.scheduleUpTo = -1;
    // Each seat's song as its device plays it: the world's time less its lag.
    const at = plays.map((s) => songNow(world, g) - s.lagMs / 1000);
    const len = beatLength(g);
    const window = t.goodMs / 1000;
    let hitMask = 0;
    let judgedNow = 0;
    if (g.resetNotes) for (const s of plays) s.notes = 0;
    for (const a of q.archetypes()) {
      const n = a.column(Note);
      const count = a.count;
      if (g.resetNotes) {
        n.live.fill(0);
        n.errorMs.fill(0);
        for (let i = 0; i < count; i++) {
          const skip = n.beat[i]! < g.fromBeat;
          n.judged[i] = skip ? 5 : 0;
          if (!skip) plays[n.seat[i]!]!.notes += 1;
        }
      }
      if (g.phase !== Phase.Play) continue;
      // Scheduling: notes up to `upTo` beats come onto the highway (og.MusicBeat, or a start's first ones).
      if (upTo >= 0) {
        for (let i = 0; i < count; i++) {
          if (!n.live[i] && !n.judged[i] && n.beat[i]! <= upTo) {
            n.live[i] = 1;
            if (n.seat[i] === 0) g.scheduled += 1;
          }
        }
      }
      if (live) {
        // Judging each pressed lane: its seat's nearest unjudged note within the Good window.
        for (let k = 0; k < plays.length * LANES; k++) {
          if (!(mask & (1 << k))) continue;
          const seat = Math.floor(k / LANES);
          const lane = k % LANES;
          const s = plays[seat]!;
          if (s.failed && plays.length > 1) continue;
          let best = -1;
          let bestErr = Infinity;
          for (let i = 0; i < count; i++) {
            if (!n.live[i] || n.judged[i] || n.lane[i] !== lane || n.seat[i] !== seat) continue;
            const err = heard[seat]! - (g.songStart + n.beat[i]! * len);
            if (Math.abs(err) <= window && Math.abs(err) < Math.abs(bestErr)) {
              best = i;
              bestErr = err;
            }
          }
          if (best < 0) continue;
          const ms = bestErr * 1000;
          const j = Math.abs(ms) <= t.perfectMs ? Judgement.Perfect : Math.abs(ms) <= t.greatMs ? Judgement.Great : Judgement.Good;
          n.judged[best] = j + 1;
          n.live[best] = 0;
          n.errorMs[best] = ms;
          judge(world, g, plays, seat, t, j, lane, ms);
          hitMask |= 1 << k;
          judgedNow += 1;
        }
        // Misses: unjudged notes past the Good window (a seat out of a versus misses no more).
        for (let i = 0; i < count; i++) {
          if (!n.live[i] || n.judged[i]) continue;
          const seat = n.seat[i]!;
          if (at[seat]! - (g.songStart + n.beat[i]! * len) > window) {
            n.judged[i] = Judgement.Miss + 1;
            n.live[i] = 0;
            if (plays[seat]!.failed && plays.length > 1) continue;
            judge(world, g, plays, seat, t, Judgement.Miss, n.lane[i]!, 0);
            judgedNow += 1;
          }
        }
      }
      // The highways: live notes fall to the hit line, missed ones fall on, dimmed, and fade.
      const speed = (g.topY - g.hitY) / (t.scrollBeats * len);
      for (let i = 0; i < count; i++) {
        const missed = n.judged[i] === Judgement.Miss + 1;
        if (!n.live[i] && !missed) continue;
        const seat = n.seat[i]!;
        const dt = g.songStart + n.beat[i]! * len - at[seat]!;
        const y = g.hitY + dt * speed;
        if (y > g.topY + 0.4 || y < g.hitY - 3) continue;
        const lane = n.lane[i]!;
        const c = LANE_COLOURS[lane]!;
        const out = plays[seat]!.failed && plays.length > 1;
        must(og.render.sprite(world, { texture: g.noteTexture as unknown as never, position: { x: highwayX(g, seat) + (lane - 1) * g.laneW, y, z: 0 }, width: g.laneW * 0.86, height: g.laneW * 0.36, color: missed || out ? { r: 0.45, g: 0.45, b: 0.5 } : c, opacity: missed ? Math.max(0, 0.5 + dt) : out ? 0.35 : 1, layer: 6 }));
      }
    }
    if (g.resetNotes) g.resetNotes = false;
    if (hitMask !== g.hitMask) g.hitMask = hitMask;
    if (g.phase !== Phase.Play || g.pausedAt >= 0) return;
    // The count-in: its numbers over the middle lane (of each highway).
    if (now < g.countTo) {
      const left = Math.ceil((g.countTo - now) / len - 1e-6);
      if (left !== g.countShown && left >= 1 && left <= 4) {
        g.countShown = left;
        for (let seat = 0; seat < plays.length; seat++) {
          must(og.render.floatingText(world, { text: `${left}`, position: { x: highwayX(g, seat), y: g.hitY + 3.2, z: 0 }, size: 0.9, color: { r: 0.85, g: 0.95, b: 1 }, rise: 0.3, lifetime: len * 0.9 }));
        }
      }
    }
    // The end, for every seat together: a beat after the last note is judged (every seat still in has judged its chart).
    if (judgedNow > 0 && g.endAt === 0 && plays.every((s) => s.failed || s.judged >= s.notes) && !plays.every((s) => s.failed)) {
      g.cleared = true;
      g.endAt = now + 1.5;
      g.sfx |= Sound.Clear;
    }
    if (g.endAt > 0 && now >= g.endAt && !g.over) g.over = true;
  },
);

/** A note judged for a seat: its tally, the words over its lane, the feel; a groove run out ends the song (in versus, once every seat's has). */
function judge(world: World, g: GameV, plays: readonly SeatV[], seat: number, t: TuneV, j: number, lane: number, ms: number): void {
  const s = plays[seat]!;
  s.judged += 1;
  s.lastJudgement = j;
  s.lastLane = lane;
  const x = highwayX(g, seat) + (lane - 1) * g.laneW;
  const y = g.hitY + 0.9;
  if (j === Judgement.Miss) {
    s.miss += 1;
    if (s.combo >= 10) g.sfx |= Sound.Break;
    else g.sfx |= Sound.Miss;
    s.combo = 0;
    // (f32 steps: what is left under a thousandth is empty.)
    const left = s.groove - t.grooveLoss;
    s.groove = left < 1e-3 ? 0 : left;
    must(og.render.floatingText(world, { text: say(world, "judge.miss"), position: { x, y, z: 0 }, size: 0.42, color: JUDGE_COLOURS[3]!, rise: 0.6, lifetime: 0.6 }));
    must(og.camera.shake(world, 0.18 * feel(world)));
    if (s.groove <= 0 && !s.failed) {
      s.failed = true;
      if (plays.every((p) => p.failed)) {
        g.endAt = worldTime(world) + 1.2;
        g.sfx |= Sound.Fail;
        g.stopBand = true;
      }
    }
    return;
  }
  s.hits += 1;
  s.combo += 1;
  s.maxCombo = Math.max(s.maxCombo, s.combo);
  const multiplier = Math.min(4, 1 + Math.floor(s.combo / t.comboStep));
  const points = j === Judgement.Perfect ? t.perfectPoints : j === Judgement.Great ? t.greatPoints : t.goodPoints;
  s.score += points * multiplier;
  if (j === Judgement.Perfect) s.perfect += 1;
  else if (j === Judgement.Great) s.great += 1;
  else s.good += 1;
  s.groove = Math.min(1, s.groove + t.grooveGain * (j === Judgement.Perfect ? 1 : j === Judgement.Great ? 0.7 : 0.4));
  s.lastOffsetMs = ms;
  const word = say(world, `judge.${JUDGEMENTS[j]}`);
  must(og.render.floatingText(world, { text: word, position: { x, y, z: 0 }, size: j === Judgement.Perfect ? 0.46 : 0.4, color: JUDGE_COLOURS[j]!, rise: 0.7, lifetime: 0.55 }));
  if (j !== Judgement.Perfect) {
    must(og.render.floatingText(world, { text: say(world, ms < 0 ? "judge.early" : "judge.late"), position: { x, y: y - 0.45, z: 0 }, size: 0.26, color: { r: 0.75, g: 0.85, b: 0.95 }, rise: 0.4, lifetime: 0.45 }));
  }
  // A combo of every comboStep raises the multiplier: a punch on the beat.
  const k = feel(world);
  if (s.combo % t.comboStep === 0 && s.combo <= t.comboStep * 3) must(og.camera.punch(world, 0.03 * k));
  must(og.camera.kick(world, { x: 0, y: -1, z: 0 }, 0.04 * k));
}

/** The song's og.Music clock follows Game.songStart and Song's tempo (the Ui and the commands move it). */
export const Clock = defineSystem("Clock", { reads: [Game], writes: [og.Music], after: ["Ui"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  for (const a of q.archetypes()) {
    const m = a.column(og.Music);
    for (let i = 0; i < a.count; i++) {
      if (m.start[i] !== g.songStart) m.start[i] = g.songStart;
      if (m.bpm[i] !== g.bpm) m.bpm[i] = g.bpm;
    }
  }
});

/**
 * The band: on the last beat of each bar og.MusicBeat brings, the stems of
 * the next bar start on its bar line (startAt, to the sample); a start or a
 * resume asks for the count-in's four clicks and the stems sounding at the
 * beat it starts on, each from where it would be (startOffset); a
 * calibration clicks every beat; a pause, a fail or the title stops them.
 */
export const Band = defineSystem("Band", { reads: [og.MusicBeat, Tuning], writes: [Game, Stem, og.AudioSource], after: ["Play"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  const len = beatLength(g);
  // A stop goes first, alone: a start asked for in the same tick waits a tick (a source stopped
  // and started in one tick would not restart).
  if (g.stopBand) {
    g.stopBand = false;
    for (const a of q.archetypes()) a.column(og.AudioSource).playing.fill(0);
    return;
  }
  // Each stem to start: when (world time) and how far into it (seconds).
  const starts = new Map<string, { at: number; offset: number }>();
  if (g.bandFrom > 0) {
    // A start or a resume: the count-in's clicks, then the stems sounding at Game.bandBeat, each
    // from where it would be by then: a bar's own from its bar line, a two-bar one from the
    // odd bar before.
    const clicks = Math.min(4, lazyTuning(world).countInBeats);
    for (let k = 4 - clicks; k < 4; k++) starts.set(`click${k}`, { at: g.countTo - (4 - k) * len, offset: 0 });
    const bar = g.bandFrom;
    if (bar % 2 === 0) {
      for (const p of partsAt(bar - 1, g.bars)) if (p !== "drums") starts.set(p, { at: g.countTo, offset: (g.bandBeat - (bar - 2) * 4) * len });
    }
    for (const p of partsAt(bar, g.bars)) starts.set(p, { at: g.countTo, offset: (g.bandBeat - (bar - 1) * 4) * len });
    g.nextBar = bar + 1;
    g.bandFrom = 0;
  }
  for (const e of world.events(og.MusicBeat)) {
    if (g.phase === Phase.Calibrate) {
      starts.set(`click${e.beat % 4}`, { at: e.time + len, offset: 0 });
    } else if (g.phase === Phase.Play && g.pausedAt < 0 && e.beat === 4 && e.bar + 1 === g.nextBar) {
      for (const p of partsAt(g.nextBar, g.bars)) starts.set(p, { at: e.time + len, offset: 0 });
      g.nextBar += 1;
    }
  }
  if (starts.size === 0) return;
  for (const a of q.archetypes()) {
    const stem = a.column(Stem);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const part = PARTS[stem.part[i]!]!;
      const start = starts.get(part);
      if (start === undefined) continue;
      src.startAt[i] = start.at;
      src.startOffset[i] = start.offset;
      src.playing[i] = 1;
      stem.starts[i] = stem.starts[i]! + 1;
    }
  }
});

/** The receptors: a press squashes its lane's, a hit flashes it in the lane's colour and bursts sparks; the lanes glow on each beat. */
export const Lanes = defineSystem("Lanes", { reads: [Game, og.MusicBeat, Prefs], writes: [Receptor, og.Transform, og.Sprite, og.ParticleSystem], after: ["Band"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  let beat = false;
  for (const _ of world.events(og.MusicBeat)) beat = true;
  const calm = (() => {
    const p = world.resource(Prefs);
    return p.ok && p.value.reducedMotion;
  })();
  for (const a of q.archetypes()) {
    const r = a.column(Receptor);
    const p = a.column(og.Transform).position;
    const sprite = a.column(og.Sprite);
    const fx = a.column(og.ParticleSystem);
    for (let i = 0; i < a.count; i++) {
      const lane = r.lane[i]!;
      const bit = 1 << (r.seat[i]! * LANES + lane);
      const me = a.entity(i);
      p[i * 3] = highwayX(g, r.seat[i]!) + (lane - 1) * g.laneW;
      p[i * 3 + 1] = g.hitY;
      const shown = g.phase === Phase.Play || g.phase === Phase.Calibrate ? 1 : 0;
      if (sprite.visible[i] !== shown) sprite.visible[i] = shown;
      if (g.pressMask & bit) {
        r.presses[i] = r.presses[i]! + 1;
        must(og.tween.squash(world, { entity: me, impact: { x: 1.12, y: 0.82 }, duration: 0.12 }));
      }
      if (g.hitMask & bit) {
        must(og.tween.flash(world, { entity: me, color: LANE_COLOURS[lane]!, duration: 0.12 }));
        fx.visible[i] = 1;
        fx.startTick[i] = world.tick;
      }
      // The ring breathes with the beat (not with reduced motion).
      const o = sprite.opacity[i]!;
      const target = beat && !calm ? 1 : Math.max(0.55, o * 0.94);
      if (Math.abs(target - o) > 1e-3) sprite.opacity[i] = target;
    }
  }
});

/** The lanes' glows follow the layout: each under its lane of its seat's highway. */
export const Glows = defineSystem("Glows", { reads: [Game, Glow], writes: [og.Transform, og.Sprite], after: ["Lanes"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  const width = (1.4 * g.laneW) / 1.5;
  for (const a of q.archetypes()) {
    const glow = a.column(Glow);
    const p = a.column(og.Transform).position;
    const sprite = a.column(og.Sprite);
    for (let i = 0; i < a.count; i++) {
      const x = highwayX(g, glow.seat[i]!) + (glow.lane[i]! - 1) * g.laneW;
      if (p[i * 3] !== x) p[i * 3] = x;
      if (Math.abs(sprite.width[i]! - width) > 1e-6) sprite.width[i] = width;
    }
  }
});

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource], after: ["Glows"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.sfx === 0) return;
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
        src.startAt[i] = 0;
        sfx.plays[i] = sfx.plays[i]! + 1;
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
