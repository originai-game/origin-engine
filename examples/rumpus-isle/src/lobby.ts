// The lobby's seats: joining, leaving, readiness and the keyboard's colour
// choice, read from the six sources each tick (src/input.ts). A pad's A on
// a free pad joins it; once seated, its A presses what its focus ring is on
// (og.ui.setPlayers: the colours, Ready, Start), so the screens (src/ui.ts)
// take its colour and readiness from Response.pad. A keyboard half has no
// ring of its own (og.ui.setPlayers rings are the pads'), so its keys pick
// its colour (left and right) and its readiness (Space or Enter) here.
//
// Drop-in: the scoreboard between rounds takes new players too (they play
// from the next round, with no points yet).

import { og, defineSystem } from "@origingame/engine";
import type { Value } from "@origingame/engine";
import { Ask, COLOURS, Game, MAX_PLAYERS, Phase, Sfx, Sound, Source, SOURCES } from "./components.ts";
import { intentOf } from "./input.ts";
import { joined, put, seat, seats } from "./seats.ts";
import { must } from "./util.ts";

type GameV = Value<typeof Game>;

/** Ticks of "Starting…" once everyone seated is ready. */
export const LAUNCH_TICKS = 100;

/** The first colour nobody else wears, from `from` on. */
export function freeColour(g: GameV, k: number, from: number, step = 1): number {
  const taken = seats(g).filter((s, o) => o !== k && s.joined).map((s) => s.colour);
  for (let i = 0; i < COLOURS.length; i++) {
    const c = (((from + i * step) % COLOURS.length) + COLOURS.length) % COLOURS.length;
    if (!taken.includes(c)) return c;
  }
  return from;
}

/** Seats `source` in the first free seat; false when the isle is full. */
export function sit(g: GameV, source: number, sfx: Value<typeof Sfx>): boolean {
  const all = seats(g);
  const k = all.findIndex((s) => !s.joined);
  if (k < 0) return false;
  put(g, k, { joined: true, source, colour: freeColour(g, k, k), ready: false, points: 0, place: 0, gained: 0, alive: false });
  g.joins += 1;
  g.launch = 0;
  sfx.bits |= Sound.Join;
  return true;
}

export function stand(g: GameV, k: number, sfx: Value<typeof Sfx>): void {
  put(g, k, { joined: false, source: Source.None, ready: false });
  g.leaves += 1;
  g.launch = 0;
  sfx.bits |= Sound.Leave;
}

export const Lobby = defineSystem("Lobby", { reads: [og.Input, og.Gamepad], writes: [Game, Sfx] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  if ((g.phase !== Phase.Lobby && g.phase !== Phase.Scores) || g.settingsOpen || g.paused) return;
  const sfx = must(world.resource(Sfx));
  const input = must(world.resource(og.Input));
  const pads = must(world.resource(og.Gamepad));
  for (let src = 0; src < SOURCES; src++) {
    const at = seats(g).findIndex((s) => s.joined && s.source === src);
    const pad = src <= Source.Pad3;
    if (pad && !pads.connected(src)) {
      // A pad unplugged in the lobby gives its seat up.
      if (at >= 0 && g.phase === Phase.Lobby) stand(g, at, sfx);
      continue;
    }
    const i = intentOf(src, input, pads);
    if (at < 0) {
      if (i.act) sit(g, src, sfx);
      continue;
    }
    // Leaving: in the lobby always; between rounds only while two others stay.
    if (i.leave && (g.phase === Phase.Lobby || joined(g).length > 2)) {
      stand(g, at, sfx);
      continue;
    }
    if (g.phase !== Phase.Lobby || pad) continue;
    const s = seat(g, at);
    if (i.left || i.right) {
      put(g, at, { colour: freeColour(g, at, s.colour + (i.right ? 1 : -1), i.right ? 1 : -1), ready: false });
      sfx.bits |= Sound.Join;
    }
    if (i.act) {
      put(g, at, { ready: !s.ready });
      sfx.bits |= s.ready ? Sound.Leave : Sound.Ready;
    }
  }
  if (g.phase === Phase.Lobby) {
    const who = joined(g);
    const all = seats(g);
    if (who.length >= 2 && who.every((k) => all[k]!.ready)) {
      g.launch += 1;
      if (g.launch >= LAUNCH_TICKS) {
        g.launch = 0;
        g.ask = Ask.Match;
      }
    } else g.launch = 0;
  }
  void MAX_PLAYERS;
});
