// The match: three rounds of different minigames on the isle, each a few
// seconds of count-in, then play, then a beat to see who won before the
// scoreboard; the podium after the third.
//
// - Bump Ring (Round.Bump): push the others into the sea; the last one on the
//   island wins. At the whistle, those still on are ranked by how near the
//   middle they stand.
// - Berry Rush (Round.Berry): berries come up on the island (a gold one is
//   worth three); the most when time runs out wins. A critter in the sea
//   drops a berry and comes back after a moment.
// - Crumble Tiles (Round.Crumble): the island sinks away under a grid of
//   tiles; a tile stood on cracks and drops; the last one standing wins.
//
// Places give points (3, 2, 1, 0; a tie shares the place). Rules runs the
// clock and the places; Tiles and Berries run their round's things over their
// own archetypes, reading where the critters are from Game (seats.ts).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { BERRIES, GRID, ISLAND_PARK_Y, PARK_Y, PODIUM, parkOf, startOf } from "./arena.ts";
import { Ask, Berry, Critter, Fx, Game, Phase, Round, ROUNDS, Sfx, Sound, Tile, Tuning } from "./components.ts";
import { joined, put, seat, seats } from "./seats.ts";
import { must, named, yawQuat } from "./util.ts";

type GameV = Value<typeof Game>;
type TuningV = Value<typeof Tuning>;

export const INTRO_TICKS = 240;
export const OVER_TICKS = 100;
/** Points for first, second, third and fourth. */
export const POINTS = [3, 2, 1, 0] as const;

function roundSeconds(t: TuningV, round: number): number {
  return round === Round.Bump ? t.bumpSeconds : round === Round.Berry ? t.berrySeconds : t.crumbleSeconds;
}

/** Puts seat k's critter at a point, still; asleep (no gravity) when parked. */
function place(world: World, k: number, x: number, y: number, z: number, yaw: number, awake: boolean): void {
  const e = named(world, `p${k}`);
  must(world.set(e, og.Transform, { position: { x, y, z }, quaternion: yawQuat(yaw) }));
  must(world.set(e, og.Velocity, { linear: { x: 0, y: 0, z: 0 }, angular: { x: 0, y: 0, z: 0 } }));
  must(world.set(e, og.RigidBody, { gravityScale: awake ? 1 : 0 }));
  must(world.set(e, Critter, { dash: 0, cooldown: 0, stun: 0, sunk: 0, yaw }));
}

/** Seat k's critter off stage. */
export function park(world: World, k: number): void {
  const p = parkOf(k);
  place(world, k, p.x, p.y, p.z, 0, false);
}

/** The island up (or sunk away), the tiles laid (or under the sea), the berries and podium put away. */
function setStage(world: World, island: boolean, tiles: boolean, podium: boolean): void {
  must(world.set(named(world, "island"), og.Transform, { position: { x: 0, y: island ? 0 : ISLAND_PARK_Y, z: 0 } }));
  for (let i = 0; i < GRID; i++) {
    for (let j = 0; j < GRID; j++) {
      const e = named(world, `tile-${i}-${j}`);
      const home = must(world.get(e, Tile));
      must(world.set(e, og.Transform, { position: { x: home.homeX, y: tiles ? 0 : PARK_Y, z: home.homeZ } }));
      must(world.set(e, Tile, { crack: 0, fall: 0, down: !tiles }));
    }
  }
  for (let b = 0; b < BERRIES; b++) {
    const e = named(world, `berry-${b}`);
    must(world.set(e, og.Transform, { position: { x: b, y: PARK_Y, z: 0 } }));
    must(world.set(e, Berry, { up: false, gold: false, age: 0 }));
  }
  must(world.set(named(world, "podium"), og.Transform, { position: { x: PODIUM.x, y: podium ? 0 : PARK_Y, z: PODIUM.z } }));
}

/** The lobby: everyone off stage, the island back. */
export function toLobby(world: World, g: GameV): void {
  g.phase = Phase.Lobby;
  g.paused = g.settingsOpen = false;
  g.launch = 0;
  setStage(world, true, false, false);
  for (let k = 0; k < 4; k++) {
    put(g, k, { ready: false, alive: false, place: 0, gained: 0, berries: 0, lasted: 0 });
    // The seated ones stand on the island while they choose: they can see their colour.
    if (seat(g, k).joined) place(world, k, -3 + k * 2, 0.05, -2.6, 0, true);
    else park(world, k);
  }
}

/** The title: nobody seated. */
export function toTitle(world: World, g: GameV): void {
  for (let k = 0; k < 4; k++) put(g, k, { joined: false, source: -1, ready: false, points: 0 });
  toLobby(world, g);
  g.phase = Phase.Title;
}

/** A new match for the seated players. */
export function beginMatch(world: World, g: GameV): void {
  for (let k = 0; k < 4; k++) put(g, k, { points: 0 });
  g.matches += 1;
  beginRound(world, g, 0);
}

/** Round r's count-in: the stage set, every seated critter at its start. */
export function beginRound(world: World, g: GameV, r: number): void {
  const t = must(world.resource(Tuning));
  g.round = r;
  g.phase = Phase.Intro;
  g.ticks = INTRO_TICKS;
  g.left = roundSeconds(t, r);
  g.played = 0;
  g.outs = 0;
  g.newRecord = false;
  g.paused = g.settingsOpen = false;
  setStage(world, r !== Round.Crumble, r === Round.Crumble, false);
  const who = joined(g);
  g.standing = who.length;
  for (let k = 0; k < 4; k++) {
    put(g, k, { alive: false, place: 0, gained: 0, berries: 0, lasted: 0, ready: false });
    if (!who.includes(k)) {
      park(world, k);
      continue;
    }
    put(g, k, { alive: true });
    const s = startOf(k, r === Round.Crumble ? 2.6 : 3.4);
    place(world, k, s.x, 0.05, s.z, s.yaw, true);
  }
}

/** Ranks and scores the round, records a record, and shows it. */
function finishRound(world: World, g: GameV, t: TuningV, sfx: Value<typeof Sfx>): void {
  const who = joined(g);
  const all = seats(g);
  if (g.round === Round.Berry) {
    // Most berries first; a tie shares the place.
    for (const k of who) put(g, k, { place: 1 + who.filter((o) => all[o]!.berries > all[k]!.berries).length });
    const most = Math.max(0, ...who.map((k) => all[k]!.berries));
    if (most > g.bestBerries) {
      g.bestBerries = most;
      g.newRecord = g.writeRecords = true;
    }
  } else {
    // Those still standing: the bump ring ranks them by how near the middle they are; the tiles let them share first.
    const standing = who.filter((k) => all[k]!.alive);
    const near = (k: number) => Math.hypot(all[k]!.x, all[k]!.z);
    for (const k of standing) {
      const ahead = g.round === Round.Bump ? standing.filter((o) => near(o) < near(k) - 1e-6).length : 0;
      put(g, k, { place: 1 + ahead, lasted: g.played / 60 });
    }
    if (g.round === Round.Crumble) {
      const longest = Math.max(0, ...who.map((k) => seat(g, k).lasted));
      if (longest > g.bestCrumble + 1e-3) {
        g.bestCrumble = Math.round(longest * 10) / 10;
        g.newRecord = g.writeRecords = true;
      }
    }
  }
  for (const k of who) {
    const s = seat(g, k);
    const gained = POINTS[Math.min(3, Math.max(0, s.place - 1))]!;
    put(g, k, { gained, points: s.points + gained });
  }
  g.phase = Phase.Over;
  g.ticks = OVER_TICKS;
  sfx.bits |= Sound.Whistle;
}

/** After the scoreboard: the next round, or the podium. */
export function nextRound(world: World, g: GameV): void {
  if (g.round + 1 < ROUNDS) beginRound(world, g, g.round + 1);
  else toPodium(world, g);
}

/** The podium: the three best on its steps, the rest in front; confetti. */
export function toPodium(world: World, g: GameV): void {
  g.phase = Phase.Podium;
  g.paused = g.settingsOpen = false;
  g.shown = 0;
  setStage(world, true, false, true);
  const who = joined(g);
  const all = seats(g);
  // By points; a tie keeps the seat order, and shares the place's name in the UI.
  const order = [...who].sort((a, b) => all[b]!.points - all[a]!.points || a - b);
  order.forEach((k, i) => {
    put(g, k, { place: 1 + who.filter((o) => all[o]!.points > all[k]!.points).length, alive: true });
    if (i < 3) {
      const step = PODIUM.steps[i]!;
      place(world, k, PODIUM.x + step.x, step.y + 0.05, PODIUM.z, 0, true);
    } else place(world, k, 2.8, 0.05, 1.4, -0.4, true);
  });
  for (let k = 0; k < 4; k++) if (!who.includes(k)) park(world, k);
}

export const Rules = defineSystem(
  "Rules",
  { reads: [Tuning], writes: [Game, Sfx, Fx, Critter, og.Transform, og.Velocity, og.RigidBody, Tile, Berry, og.Rng] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const sfx = must(world.resource(Sfx));
    const fx = must(world.resource(Fx));
    // What the screens asked for (Game.ask): they build the UI, this moves the world.
    const ask = g.ask;
    g.ask = Ask.None;
    if (ask === Ask.Match || ask === Ask.Again) beginMatch(world, g);
    else if (ask === Ask.Next) nextRound(world, g);
    else if (ask === Ask.Lobby) toLobby(world, g);
    else if (ask === Ask.Title) toTitle(world, g);
    if (ask !== Ask.None) return;
    if (g.phase === Phase.Lobby) {
      // A player who sat down stands on the island in their colour; one who left goes off stage.
      for (let k = 0; k < 4; k++) {
        const s = seat(g, k);
        const parked = s.y < PARK_Y + 1;
        if (s.joined && parked) {
          // In a row across the back of the island, facing the camera, above the lobby's panel.
          place(world, k, -3 + k * 2, 0.05, -2.6, 0, true);
          put(g, k, { y: 0.05 });
        } else if (!s.joined && !parked) {
          park(world, k);
          put(g, k, { y: PARK_Y });
        }
      }
      return;
    }
    if (g.paused || g.settingsOpen || time.scaledDt === 0) return;
    if (g.phase === Phase.Intro) {
      if (g.ticks === 180 || g.ticks === 120 || g.ticks === 60) sfx.bits |= Sound.Beep;
      g.ticks -= 1;
      if (g.ticks === 0) {
        g.phase = Phase.Play;
        g.ticks = Math.round(roundSeconds(t, g.round) * 60);
        sfx.bits |= Sound.Go;
      }
      return;
    }
    if (g.phase === Phase.Over) {
      g.ticks -= 1;
      if (g.ticks === 0) g.phase = Phase.Scores;
      return;
    }
    if (g.phase !== Phase.Play) return;
    g.ticks -= 1;
    g.played += 1;
    g.left = g.ticks / 60;
    const who = joined(g);
    const rng = must(world.resource(og.Rng));
    for (const k of who) {
      const s = seat(g, k);
      const e = named(world, `p${k}`);
      if (g.round === Round.Berry) {
        const c = must(world.get(e, Critter));
        if (s.alive && s.y < t.seaLevel) {
          // In the sea: a berry lost, back on the island in a moment.
          put(g, k, { alive: false, berries: Math.max(0, s.berries - 1) });
          must(world.set(e, Critter, { sunk: 1 }));
          splash(g, fx, sfx, s.x, s.z);
        } else if (!s.alive && c.sunk > 0) {
          if (c.sunk >= t.respawnTicks) {
            const a = rng.range(0, Math.PI * 2), r = rng.range(0, 3);
            place(world, k, Math.cos(a) * r, 1.2, Math.sin(a) * r, a, true);
            put(g, k, { alive: true });
          } else must(world.set(e, Critter, { sunk: c.sunk + 1 }));
        }
        continue;
      }
      if (s.alive && s.y < t.seaLevel) {
        // Out: the last place not yet given.
        put(g, k, { alive: false, place: g.standing, lasted: g.played / 60 });
        g.standing -= 1;
        g.outs += 1;
        splash(g, fx, sfx, s.x, s.z);
      }
    }
    const over = g.round === Round.Berry ? g.ticks === 0 : g.ticks === 0 || (who.length > 1 ? g.standing <= 1 : g.standing === 0);
    if (over) finishRound(world, g, t, sfx);
  },
);

function splash(g: GameV, fx: Value<typeof Fx>, sfx: Value<typeof Sfx>, x: number, z: number): void {
  g.splashes += 1;
  fx.splash = true;
  fx.splashX = x;
  fx.splashZ = z;
  fx.shake = Math.max(fx.shake, 0.3);
  sfx.bits |= Sound.Splash;
}

/** The crumble round's tiles: one stood on cracks (it shakes), then drops into the sea. */
export const Tiles = defineSystem(
  "Tiles",
  { reads: [Tuning], writes: [Tile, og.Transform, Game, Fx, Sfx] },
  (q, time, world) => {
    const g = must(world.resource(Game));
    if (g.round !== Round.Crumble || (g.phase !== Phase.Play && g.phase !== Phase.Over) || time.scaledDt === 0) return;
    const t = must(world.resource(Tuning));
    const fx = must(world.resource(Fx));
    const sfx = must(world.resource(Sfx));
    const on = seats(g).filter((s) => s.joined && s.alive && s.y > -0.4);
    const cracking = g.phase === Phase.Play && g.played > t.graceTicks;
    for (const a of q.archetypes()) {
      const tile = a.column(Tile);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        if (tile.down[i]) {
          const y = p[i * 3 + 1]!;
          if (y <= PARK_Y) continue;
          tile.fall[i] = tile.fall[i]! + 16 * time.scaledDt;
          const ny = y - tile.fall[i]! * time.scaledDt;
          p[i * 3 + 1] = ny < -14 ? PARK_Y : ny;
          continue;
        }
        const hx = tile.homeX[i]!, hz = tile.homeZ[i]!;
        if (tile.crack[i] === 0) {
          if (cracking && on.some((s) => Math.abs(s.x - hx) < 0.78 && Math.abs(s.z - hz) < 0.78)) {
            tile.crack[i] = 1;
            g.cracks += 1;
            sfx.bits |= Sound.Crack;
          }
          continue;
        }
        tile.crack[i] = tile.crack[i]! + 1;
        const c = tile.crack[i]!;
        // A shake that grows as it goes.
        const amp = 0.015 + 0.05 * (c / t.crackTicks);
        p[i * 3] = hx + Math.sin(c * 1.9) * amp;
        p[i * 3 + 2] = hz + Math.cos(c * 2.3) * amp;
        p[i * 3 + 1] = -0.02 * (c / t.crackTicks);
        if (c >= t.crackTicks) {
          tile.down[i] = 1;
          tile.fall[i] = 0;
          p[i * 3] = hx;
          p[i * 3 + 2] = hz;
          fx.crumbs = true;
          fx.crumbsX = hx;
          fx.crumbsZ = hz;
          sfx.bits |= Sound.Crumble;
        }
      }
    }
  },
);

/** The berry round's berries: up to Tuning.berriesUp at once at random places, taken by the first critter to reach one. */
export const Berries = defineSystem(
  "Berries",
  { reads: [Tuning], writes: [Berry, og.Transform, Game, Fx, Sfx, og.Rng] },
  (q, time, world) => {
    const g = must(world.resource(Game));
    if (g.round !== Round.Berry || g.phase !== Phase.Play || time.scaledDt === 0) return;
    const t = must(world.resource(Tuning));
    const fx = must(world.resource(Fx));
    const sfx = must(world.resource(Sfx));
    const rng = must(world.resource(og.Rng));
    const all = seats(g);
    let up = 0;
    let spawned = false;
    for (const a of q.archetypes()) {
      const b = a.column(Berry);
      const tr = a.column(og.Transform);
      for (let i = 0; i < a.count; i++) if (b.up[i]) up++;
      for (let i = 0; i < a.count; i++) {
        if (!b.up[i]) {
          // One comes up a tick, while fewer than berriesUp are; the first ones after a short wait.
          if (spawned || up >= t.berriesUp || g.played < 20 || g.played % 9 !== 0) continue;
          let x = 0, z = 0;
          for (let tries = 0; tries < 8; tries++) {
            const ang = rng.range(0, Math.PI * 2), r = Math.sqrt(rng.next()) * 5.4;
            x = Math.cos(ang) * r;
            z = Math.sin(ang) * r;
            if (all.every((s) => !s.joined || Math.hypot(s.x - x, s.z - z) > 1.6)) break;
          }
          b.up[i] = 1;
          b.gold[i] = rng.chance(0.15) ? 1 : 0;
          b.age[i] = 0;
          tr.position[i * 3] = x;
          tr.position[i * 3 + 1] = 0.02;
          tr.position[i * 3 + 2] = z;
          const s = b.gold[i] ? 1.45 : 1;
          tr.scale[i * 3] = tr.scale[i * 3 + 1] = tr.scale[i * 3 + 2] = s * 0.2;
          up++;
          spawned = true;
          continue;
        }
        b.age[i] = b.age[i]! + 1;
        const age = b.age[i]!;
        const s = (b.gold[i] ? 1.45 : 1) * Math.min(1, 0.2 + age / 10);
        tr.scale[i * 3] = tr.scale[i * 3 + 1] = tr.scale[i * 3 + 2] = s;
        tr.position[i * 3 + 1] = 0.02 + 0.06 * (1 + Math.sin(age * 0.12));
        const q2 = yawQuat(age * 0.03);
        tr.quaternion[i * 4 + 1] = q2.y;
        tr.quaternion[i * 4 + 3] = q2.w;
        const bx = tr.position[i * 3]!, bz = tr.position[i * 3 + 2]!;
        const taker = all.findIndex((st) => st.joined && st.alive && st.y > -0.3 && Math.hypot(st.x - bx, st.z - bz) < 0.8);
        if (taker >= 0) {
          const gain = b.gold[i] ? 3 : 1;
          put(g, taker, { berries: all[taker]!.berries + gain });
          all[taker]!.berries += gain;
          g.berriesTaken += 1;
          sfx.bits |= b.gold[i] ? Sound.Gold : Sound.Berry;
          fx.sparkle = true;
          fx.sparkleX = bx;
          fx.sparkleZ = bz;
          b.up[i] = 0;
          tr.position[i * 3 + 1] = PARK_Y;
          up--;
        } else if (b.gold[i] && age > 300) {
          // A gold berry does not wait for ever.
          b.up[i] = 0;
          tr.position[i * 3 + 1] = PARK_Y;
          up--;
        }
      }
    }
  },
);
