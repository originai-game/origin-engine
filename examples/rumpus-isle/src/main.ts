// Rumpus Isle (3D profile): a party game for two to four players on one
// screen. Round critters on a little island bump each other about in three
// short minigames (src/rules.ts): Bump Ring, push the others into the sea;
// Berry Rush, take the most berries before the whistle; Crumble Tiles, the
// last one on a grid of tiles that drop once stood on. Places give points,
// a scoreboard follows each round, and a podium the third.
//
// Up to four players sit down in the lobby (src/lobby.ts): any of the four
// pads, and two halves of the keyboard (WASD and Space, the arrows and
// Enter). Each critter is a dynamic body (src/critters.ts), pushed towards
// what its player asks, with a dash; the bumps between them are physics plus
// a shove by how fast they met.
//
// The look (GameDesc.look, assets/looks/rumpus-isle.ogt: the miniature
// diorama preset with the rumpus skin) sets the frame before `start`; the
// scene (assets/scenes/main.ogt) is the island, the islets and the sun;
// `start` builds the rest (src/arena.ts), the voices and the music
// (src/sound.ts), and the string tables.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { buildArena } from "./arena.ts";
import { Bumps, Drive } from "./critters.ts";
import { Ask, Fx, Game, MAX_PLAYERS, Phase, Prefs, Round, Sfx, Source, Tuning } from "./components.ts";
import { Effects, Tags, View } from "./feel.ts";
import { Lobby, sit } from "./lobby.ts";
import { applyPrefs, LANGUAGES, setLanguage } from "./prefs.ts";
import { beginMatch, beginRound, nextRound, Berries, Rules, Tiles, toLobby, toPodium, toTitle } from "./rules.ts";
import { Answers, Saves } from "./save.ts";
import { joined, put, seat } from "./seats.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { Ui } from "./ui.ts";
import { must, named } from "./util.ts";

function start(world: World): void {
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const g = must(world.resource(Game));
  g.font = must(og.load.font(world, "skins/rumpus/LilitaOne-Regular.ttf"));
  buildArena(world);
  spawnVoices(world);
  applyPrefs(world, must(world.resource(Prefs)));
  toTitle(world, g);
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

const SOURCE_NAMES: Record<string, number> = { pad0: Source.Pad0, pad1: Source.Pad1, pad2: Source.Pad2, pad3: Source.Pad3, wasd: Source.KeysLeft, arrows: Source.KeysRight };

function sourceArg(s: string | undefined): number {
  const v = SOURCE_NAMES[s ?? ""];
  if (v === undefined) throw new Error(`a source is one of ${Object.keys(SOURCE_NAMES).join(", ")}`);
  return v;
}

export default og.game({
  look: "looks/rumpus-isle.ogt",
  resources: [Game, Prefs, Tuning, Fx, Sfx],
  systems: [Answers, Lobby, Ui, Rules, Drive, Bumps, Tiles, Berries, View, Tags, Effects, Voices, Saves],
  render: { antialias: "taa" },
  start,
  commands: {
    seat: (world, args) => {
      // seat:<source>[:<colour>]: sits a source down in the lobby, as its join press does.
      const g = gameOf(world);
      if (g.phase === Phase.Title) toLobby(world, g);
      const src = sourceArg(args[0]);
      if (seat(g, 0).source === src && seat(g, 0).joined) return;
      if (!sit(g, src, must(world.resource(Sfx)))) throw new Error("the isle is full (four players)");
      if (args[1] !== undefined) {
        const k = [0, 1, 2, 3].find((i) => seat(g, i).joined && seat(g, i).source === src)!;
        put(g, k, { colour: Number(args[1]) });
      }
    },
    lobby: (world) => toLobby(world, gameOf(world)),
    match: (world) => {
      const g = gameOf(world);
      if (joined(g).length < 2) throw new Error("a match needs two seated players: seat:<source> first");
      beginMatch(world, g);
    },
    round: (world, args) => {
      // round:<1-3>: that round's count-in now, with the points as they are.
      const g = gameOf(world);
      const r = Number(args[0]) - 1;
      if (!(r >= 0 && r <= Round.Crumble)) throw new Error("write round:<1-3>");
      if (joined(g).length < 1) throw new Error("seat a player first: seat:<source>");
      beginRound(world, g, r);
    },
    go: (world) => {
      // Skips the count-in.
      const g = gameOf(world);
      if (g.phase === Phase.Intro) g.ticks = 1;
    },
    finish: (world) => {
      const g = gameOf(world);
      if (g.phase !== Phase.Play) throw new Error("finish ends a round in play");
      g.ticks = 1;
    },
    next: (world) => nextRound(world, gameOf(world)),
    podium: (world, args) => {
      const g = gameOf(world);
      // podium[:p0,p1,p2,p3]: the points to stand by.
      if (args[0] !== undefined) args[0].split(",").forEach((v, k) => k < MAX_PLAYERS && put(g, k, { points: Number(v) }));
      toPodium(world, g);
    },
    knock: (world, args) => {
      // knock:<seat>: drops that critter into the sea (a quick out for scripts).
      const k = Number(args[0]);
      const e = named(world, `p${k}`);
      must(world.set(e, og.Transform, { position: { x: 9, y: -1, z: 0 } }));
    },
    pause: (world) => {
      gameOf(world).paused = true;
    },
    settings: (world) => {
      gameOf(world).settingsOpen = true;
    },
    title: (world) => {
      gameOf(world).ask = Ask.Title;
    },
    language: (world, args) => {
      const code = args[0] ?? "";
      if (!LANGUAGES.some((l) => l.code === code)) throw new Error(`write language:<${LANGUAGES.map((l) => l.code).join("|")}>`);
      setLanguage(world, must(world.resource(Prefs)), code);
    },
    records: (world, args) => {
      const g = gameOf(world);
      g.bestBerries = Number(args[0] ?? 0);
      g.bestCrumble = Number(args[1] ?? 0);
    },
  },
  commandHelp: {
    seat: { usage: "<pad0|pad1|pad2|pad3|wasd|arrows>[:<colour 0-5>]", doc: "Sit a source down in the lobby (from the title too), as its join press does." },
    lobby: { usage: "", doc: "The lobby now, the seated players kept." },
    match: { usage: "", doc: "Start a match for the seated players (two or more)." },
    round: { usage: "<1-3>", doc: "That round's count-in now, the points as they are." },
    go: { usage: "", doc: "Skip the count-in." },
    finish: { usage: "", doc: "End the round in play now (the whistle)." },
    next: { usage: "", doc: "The next round, or the podium after the third, as the scoreboard's button does." },
    podium: { usage: "[p0,p1,p2,p3]", doc: "The podium now, with these points if given." },
    knock: { usage: "<seat 0-3>", doc: "Drop that seat's critter into the sea." },
    pause: { usage: "", doc: "Pause, as Escape or a pad's Start does." },
    settings: { usage: "", doc: "Open the settings." },
    title: { usage: "", doc: "Back to the title (everyone stands up)." },
    language: { usage: "<en|fr>", doc: "Switch the language as the settings do." },
    records: { usage: "<berries> <seconds>", doc: "Set the isle's records (for frames)." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    const prefs = world.resource(Prefs);
    if (!game.ok || !prefs.ok) return null;
    const g = game.value;
    const who = joined(g);
    return {
      phase: ["title", "lobby", "intro", "play", "over", "scores", "podium"][g.phase],
      round: g.round + 1,
      left: Math.round(g.left * 10) / 10,
      players: who.length,
      standing: g.standing,
      // A player falling into the sea is a hit for og polish (og-build §5): `hits` counts the bumps.
      hits: g.bumps,
      splashes: g.splashes,
      berries: who.map((k) => seat(g, k).berries),
      points: who.map((k) => seat(g, k).points),
      places: who.map((k) => seat(g, k).place),
      paused: g.paused,
      language: LANGUAGES[prefs.value.language]!.code,
      a11yUnnamed: g.a11yUnnamed,
    };
  },
});
