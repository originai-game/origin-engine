// Three-Lane Beat's systems, in tick order after the screens (src/ui.ts):
// Clock (the song's og.Music clock follows Game.songStart), Play (presses,
// judgements, misses, the notes' scheduling by og.MusicBeat, drawing the
// highway), Band (the song's stems started on their bar lines by
// og.MusicBeat, the count-in's clicks), Lanes (the receptors' flash,
// squash and burst, the lanes' glow on the beat) and Voices.
//
// Time is the world's: tick / tick rate, the clock og.music and og.audio
// keep, so a stem started at `songStart + bar × barLength` sounds on its
// bar line to the sample. A press is judged at the time it happened (its
// tick's time and its event's `at` within the tick) less the output's
// latency (og.audio.outputLatency) and the calibrated offset
// (Prefs.offsetMs) against its note's exact time.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Judgement, JUDGEMENTS, LANES, Note, Phase, Prefs, Receptor, Sfx, Sound, Stem, Tuning } from "./components.ts";
import { say } from "./ui.ts";

type GameV = Value<typeof Game>;
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

/** The world point under a CSS pixel, through the orthographic camera. */
export function screenToWorld(world: World, px: number, py: number): { x: number; y: number } | null {
  const cam = world.resource(og.Camera);
  const vp = og.ui.viewport(world);
  if (!cam.ok || vp.height <= 0) return null;
  const scale = vp.height / cam.value.height;
  const at = cam.value.position;
  return { x: at.x + (px - vp.width / 2) / scale, y: at.y - (py - vp.height / 2) / scale };
}

/** Each lane's keys and pad buttons. */
const LANE_KEYS: readonly (readonly string[])[] = [["KeyA", "ArrowLeft", "KeyJ"], ["KeyS", "ArrowDown", "KeyK", "Space"], ["KeyD", "ArrowRight", "KeyL"]];
const LANE_PADS: readonly (readonly string[])[] = [["left", "x"], ["down", "a"], ["right", "b"]];

/** The lane a finger at a CSS pixel is on, or -1 off the highway. */
function laneAt(world: World, g: GameV, x: number, y: number): number {
  const w = screenToWorld(world, x, y);
  if (w === null || w.y > g.hitY + 3.5) return -1;
  const lane = Math.round(w.x / g.laneW) + 1;
  return lane >= 0 && lane < LANES && Math.abs(w.x - (lane - 1) * g.laneW) <= g.laneW * 0.6 ? lane : -1;
}

/**
 * The lanes pressed this tick, as a mask (keys, the pad, fingers on the
 * highway, the HUD's zones), and when within the tick each lane's first
 * press happened (`within[lane]`, in ticks: og.KeyDown's and
 * og.PointerDown's `at`, from the host's event time; 0 for the pad, which
 * the shell polls once a frame, and for the `hit` command).
 */
function pressedLanes(world: World, g: GameV, inp: Value<typeof og.Input>, gp: Value<typeof og.Gamepad>, ptr: Value<typeof og.Pointers>, within: number[]): number {
  let mask = 0;
  within.fill(0);
  const stamp = (lane: number, at: number) => {
    within[lane] = mask & (1 << lane) ? Math.min(within[lane]!, at) : at;
    mask |= 1 << lane;
  };
  for (const e of world.events(og.KeyDown)) {
    if (e.repeat) continue;
    const lane = LANE_KEYS.findIndex((keys) => keys.includes(e.key));
    if (lane >= 0 && inp.pressed(e.key)) stamp(lane, e.at);
  }
  for (const e of world.events(og.PointerDown)) {
    const lane = ptr.pressed(e.id) ? laneAt(world, g, e.x, e.y) : -1;
    if (lane >= 0) stamp(lane, e.at);
  }
  for (let lane = 0; lane < LANES; lane++) {
    if (mask & (1 << lane)) continue;
    if (LANE_KEYS[lane]!.some((k) => inp.pressed(k)) || LANE_PADS[lane]!.some((b) => gp.pressed(b as never))) stamp(lane, 0);
  }
  for (let id = 0; id < ptr.count(); id++) {
    if (!ptr.pressed(id)) continue;
    const at = ptr.position(id);
    const lane = laneAt(world, g, at.x, at.y);
    if (lane >= 0 && !(mask & (1 << lane))) stamp(lane, 0);
  }
  if (g.padLane >= 0) {
    if (!(mask & (1 << g.padLane))) stamp(g.padLane, 0);
    g.padLane = -1;
  }
  return mask;
}

/** Scratch for pressedLanes: each lane's press time within the tick. */
const within = [0, 0, 0];

/**
 * The play. Each tick: the presses (the count-in's taps and a calibration's
 * are measured, not judged), each pressed lane's nearest unjudged note
 * within the Good window judged by its error, unhit notes past the window
 * missed, the notes og.MusicBeat brings onto the highway, the highway
 * drawn, and the song's end (cleared a beat after its last note, failed
 * when the groove runs out).
 */
export const Play = defineSystem(
  "Play",
  { reads: [og.Input, og.Gamepad, og.Pointers, og.KeyDown, og.PointerDown, og.Camera, og.Audio, og.MusicBeat, Tuning, Prefs], writes: [Game, Note], after: ["Clock"] },
  (q, _time, world) => {
    const game = world.resource(Game);
    const input = world.resource(og.Input);
    const gamepad = world.resource(og.Gamepad);
    const pointers = world.resource(og.Pointers);
    const prefs = world.resource(Prefs);
    if (!game.ok || !input.ok || !gamepad.ok || !pointers.ok || !prefs.ok) return;
    const g = game.value;
    const t = lazyTuning(world);
    const now = worldTime(world);
    // What the player hears is the world's time less the output's latency (0 headless); the
    // calibrated offset is the rest of the chain (the display, the hand).
    const latency = og.audio.outputLatency(world);
    const offset = prefs.value.offsetMs / 1000;
    const live = g.phase === Phase.Play && !g.held && g.pausedAt < 0;
    const calibrating = g.phase === Phase.Calibrate && !g.settingsOpen;
    const mask = live || calibrating ? pressedLanes(world, g, input.value, gamepad.value, pointers.value, within) : 0;
    const tickLength = 1 / 60;
    if (mask !== g.pressMask) g.pressMask = mask;
    if (mask !== 0) {
      g.taps += 1;
      for (let lane = 0; lane < LANES; lane++) if (mask & (1 << lane)) g.sfx |= Sound.Tap0 << lane;
    }
    // Calibration: each tap's error from the nearest click (every beat of the clock).
    if (calibrating && mask !== 0 && g.calTaps < 8 && now > g.songStart) {
      const len = beatLength(g);
      let first = 0;
      while (!(mask & (1 << first))) first++;
      // The tap when it happened, against the click when it was heard.
      const b = (now + within[first]! * tickLength - latency - g.songStart) / len;
      const err = (b - Math.round(b)) * len * 1000;
      g.calTaps += 1;
      g.calSum += err;
    }
    // The notes og.MusicBeat brings: every note up to scrollBeats past the beat just sent, and one more
    // (a start or a resume asks for the first ones through Game.scheduleUpTo).
    let upTo = g.scheduleUpTo;
    if (g.phase === Phase.Play && g.pausedAt < 0) {
      for (const e of world.events(og.MusicBeat)) upTo = Math.max(upTo, (e.bar - 1) * 4 + (e.beat - 1) + 1 + t.scrollBeats);
    }
    if (g.scheduleUpTo !== -1) g.scheduleUpTo = -1;
    const at = songNow(world, g) - offset - latency;
    const len = beatLength(g);
    const window = t.goodMs / 1000;
    let hitMask = 0;
    let judgedNow = 0;
    for (const a of q.archetypes()) {
      const n = a.column(Note);
      const count = a.count;
      if (g.resetNotes) {
        n.live.fill(0);
        n.errorMs.fill(0);
        let playing = 0;
        for (let i = 0; i < count; i++) {
          const skip = n.beat[i]! < g.fromBeat;
          n.judged[i] = skip ? 5 : 0;
          if (!skip) playing += 1;
        }
        g.notes = playing;
      }
      if (g.phase !== Phase.Play) continue;
      // Scheduling: notes up to `upTo` beats come onto the highway (og.MusicBeat, or a start's first ones).
      if (upTo >= 0) {
        for (let i = 0; i < count; i++) {
          if (!n.live[i] && !n.judged[i] && n.beat[i]! <= upTo) {
            n.live[i] = 1;
            g.scheduled += 1;
          }
        }
      }
      // Judging each pressed lane: its nearest unjudged note within the Good window.
      if (live) {
        for (let lane = 0; lane < LANES; lane++) {
          if (!(mask & (1 << lane))) continue;
          // The press when it happened within the tick, not the tick's start.
          const pressed = at + within[lane]! * tickLength;
          let best = -1;
          let bestErr = Infinity;
          for (let i = 0; i < count; i++) {
            if (!n.live[i] || n.judged[i] || n.lane[i] !== lane) continue;
            const err = pressed - (g.songStart + n.beat[i]! * len);
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
          judge(world, g, t, j, lane, ms);
          hitMask |= 1 << lane;
          judgedNow += 1;
        }
        // Misses: unjudged notes past the Good window.
        for (let i = 0; i < count; i++) {
          if (!n.live[i] || n.judged[i]) continue;
          if (at - (g.songStart + n.beat[i]! * len) > window) {
            n.judged[i] = Judgement.Miss + 1;
            n.live[i] = 0;
            judge(world, g, t, Judgement.Miss, n.lane[i]!, 0);
            judgedNow += 1;
          }
        }
      }
      // The highway: live notes fall to the hit line, missed ones fall on, dimmed, and fade.
      const speed = (g.topY - g.hitY) / (t.scrollBeats * len);
      for (let i = 0; i < count; i++) {
        const missed = n.judged[i] === Judgement.Miss + 1;
        if (!n.live[i] && !missed) continue;
        const dt = g.songStart + n.beat[i]! * len - at;
        const y = g.hitY + dt * speed;
        if (y > g.topY + 0.4 || y < g.hitY - 3) continue;
        const lane = n.lane[i]!;
        const c = LANE_COLOURS[lane]!;
        must(og.render.sprite(world, { texture: g.noteTexture as unknown as never, position: { x: (lane - 1) * g.laneW, y, z: 0 }, width: g.laneW * 0.86, height: g.laneW * 0.36, color: missed ? { r: 0.45, g: 0.45, b: 0.5 } : c, opacity: missed ? Math.max(0, 0.5 + dt) : 1, layer: 6 }));
      }
    }
    if (g.resetNotes) g.resetNotes = false;
    if (hitMask !== g.hitMask) g.hitMask = hitMask;
    if (g.phase !== Phase.Play || g.pausedAt >= 0) return;
    // The count-in: its numbers over the middle lane.
    if (now < g.countTo) {
      const left = Math.ceil((g.countTo - now) / len - 1e-6);
      if (left !== g.countShown && left >= 1 && left <= 4) {
        g.countShown = left;
        must(og.render.floatingText(world, { text: `${left}`, position: { x: 0, y: g.hitY + 3.2, z: 0 }, size: 0.9, color: { r: 0.85, g: 0.95, b: 1 }, rise: 0.3, lifetime: len * 0.9 }));
      }
    }
    // The end: a beat after the last note is judged, or soon after the groove runs out.
    if (judgedNow > 0 && g.judged >= g.notes && g.endAt === 0 && !g.failed) {
      g.cleared = true;
      g.endAt = now + 1.5;
      g.sfx |= Sound.Clear;
    }
    if (g.endAt > 0 && now >= g.endAt && !g.over) g.over = true;
  },
);

function judge(world: World, g: GameV, t: TuneV, j: number, lane: number, ms: number): void {
  g.judged += 1;
  g.lastJudgement = j;
  g.lastLane = lane;
  const x = (lane - 1) * g.laneW;
  const y = g.hitY + 0.9;
  if (j === Judgement.Miss) {
    g.miss += 1;
    if (g.combo >= 10) g.sfx |= Sound.Break;
    else g.sfx |= Sound.Miss;
    g.combo = 0;
    // (f32 steps: what is left under a thousandth is empty.)
    const left = g.groove - t.grooveLoss;
    g.groove = left < 1e-3 ? 0 : left;
    must(og.render.floatingText(world, { text: say(world, "judge.miss"), position: { x, y, z: 0 }, size: 0.42, color: JUDGE_COLOURS[3]!, rise: 0.6, lifetime: 0.6 }));
    must(og.camera.shake(world, 0.18));
    if (g.groove <= 0 && !g.failed) {
      g.failed = true;
      g.endAt = worldTime(world) + 1.2;
      g.sfx |= Sound.Fail;
      g.stopBand = true;
    }
    return;
  }
  g.hits += 1;
  g.combo += 1;
  g.maxCombo = Math.max(g.maxCombo, g.combo);
  const multiplier = Math.min(4, 1 + Math.floor(g.combo / t.comboStep));
  const points = j === Judgement.Perfect ? t.perfectPoints : j === Judgement.Great ? t.greatPoints : t.goodPoints;
  g.score += points * multiplier;
  if (j === Judgement.Perfect) g.perfect += 1;
  else if (j === Judgement.Great) g.great += 1;
  else g.good += 1;
  g.groove = Math.min(1, g.groove + t.grooveGain * (j === Judgement.Perfect ? 1 : j === Judgement.Great ? 0.7 : 0.4));
  g.lastOffsetMs = ms;
  const word = say(world, `judge.${JUDGEMENTS[j]}`);
  must(og.render.floatingText(world, { text: word, position: { x, y, z: 0 }, size: j === Judgement.Perfect ? 0.46 : 0.4, color: JUDGE_COLOURS[j]!, rise: 0.7, lifetime: 0.55 }));
  if (j !== Judgement.Perfect) {
    must(og.render.floatingText(world, { text: say(world, ms < 0 ? "judge.early" : "judge.late"), position: { x, y: y - 0.45, z: 0 }, size: 0.26, color: { r: 0.75, g: 0.85, b: 0.95 }, rise: 0.4, lifetime: 0.45 }));
  }
  // A combo of every comboStep raises the multiplier: a punch on the beat.
  if (g.combo % t.comboStep === 0 && g.combo <= t.comboStep * 3) must(og.camera.punch(world, 0.03));
  must(og.camera.kick(world, { x: 0, y: -1, z: 0 }, 0.04));
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
      const me = a.entity(i);
      p[i * 3] = (lane - 1) * g.laneW;
      p[i * 3 + 1] = g.hitY;
      const shown = g.phase === Phase.Play || g.phase === Phase.Calibrate ? 1 : 0;
      if (sprite.visible[i] !== shown) sprite.visible[i] = shown;
      if (g.pressMask & (1 << lane)) {
        r.presses[i] = r.presses[i]! + 1;
        must(og.tween.squash(world, { entity: me, impact: { x: 1.12, y: 0.82 }, duration: 0.12 }));
      }
      if (g.hitMask & (1 << lane)) {
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

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource], after: ["Lanes"] }, (q, _time, world) => {
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
