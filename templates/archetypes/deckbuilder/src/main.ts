// Ashen Crown, the deckbuilder archetype (2D profile): a card roguelike in
// the line of Slay the Spire. A party marches six steps to the Ashen
// Throne: fights, a fork between a fire and one more fight, an elite, a
// fire, and the Ashen King. Each hero plays its own deck: energy each turn,
// a hand drawn from its draw pile, the discard pile shuffled back in. The
// foes show what they will do before the heroes act. A won fight offers a
// card; a fire heals or tempers (upgrades) one. The King's fall is the
// win; every hero struck to nothing is the loss.
//
// Every hero is a seat (src/seats.ts): its og.PlayerInput is all the rules
// read (src/run.ts), so the run plays alone (the Warden), two in co-op on
// one device (the Warden and the Hunter, a hand and energy each, against
// one shared fight) or two over a room in lockstep. Both views play the
// same rules: side-2d the battle side-on on an ash plain, top-down-2d the
// same battle as tokens on a war table (src/stage.ts).
//
// `start` loads the sheet, the backdrops, the cards' art, the portraits,
// the sounds and the screen documents, and spawns the heroes, the three
// pooled foes, the flying cards and a voice per sound. The rules are in
// src/run.ts, the cards in src/cards.ts, the foes and the map in
// src/foes.ts, the screens in src/ui.ts, the views in src/stage.ts and the
// state in src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { checkClips, clipOf } from "./art.ts";
import { CARD, CARDS, UPGRADED } from "./cards.ts";
import { Foe, foeName, Game, Hero, heroName, HEROES, Phase, PHASES, Sfx, Sound, Tuning } from "./components.ts";
import { prefs } from "./device.ts";
import { FOE_SLOTS, FOES, MAP } from "./foes.ts";
import { cardArt, enter, FLY_SLOTS, load, must2, partyDown, playCard, save, startRun, Run } from "./run.ts";
import { SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { heroSpot, Stage, TOP } from "./stage.ts";
import { LANGUAGES, portraits, setLanguage, Ui } from "./ui.ts";
import { Voices } from "./voices.ts";

const DARK = { r: 0.006, g: 0.004, b: 0.004 };

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must2(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must2(og.load.style(world, "styles/ashen/style.ogt"));
  const sound = (path: string) => must2(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must2(og.load.document(world, doc));

  const settings = must2(world.resource(og.RenderSettings));
  settings.environmentIntensity = 0;
  settings.toneMapped = false;
  const camera = must2(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = 12.5;
  camera.position = { x: 0, y: 0, z: 50 };
  camera.background = DARK;

  const sheet = must2(og.load.flipbook(world, "sprites/units.ogt"));
  checkClips(sheet);
  for (const c of CARDS) cardArt.push(must2(og.load.texture(world, `cards/${c.id}.png`)) as unknown as number);
  for (const name of [...HEROES, ...FOES.map((f) => f.id)]) portraits[name] = must2(og.load.texture(world, `sprites/portraits/${name}.png`)) as unknown as number;

  const back = must2(world.spawn({ name: "backdrop", position: { x: 0, y: 2, z: -1 } }));
  must(world.insert(back, og.Sprite, { texture: must2(og.load.texture(world, TOP ? "sprites/table.png" : "sprites/battlefield.png")), width: 40, height: 44, layer: 0 }));

  const unit = (name: string, clip: string, x: number, y: number) => {
    const e = must2(world.spawn({ name, position: { x, y, z: 0 } }));
    must(world.insert(e, og.Sprite, { texture: sheet.texture, width: 3, height: 3, anchor: { x: 0.5, y: 0 }, layer: 2, visible: false, sortY: TOP }));
    must(world.insert(e, og.Flipbook, { sheet: sheet.sheet, clip: clipOf(clip) }));
    return e;
  };
  for (let seat = 0; seat < SEATS; seat++) {
    const s = heroSpot(seat, 1);
    const e = unit(heroName(seat), TOP ? `${HEROES[seat]}-token` : HEROES[seat]!, s.x, s.y);
    must(world.insert(e, Hero, { seat }));
    must(world.insert(e, og.PlayerInput, { player: seat }));
  }
  for (let i = 0; i < FOE_SLOTS; i++) {
    const e = unit(foeName(i), "imp", 40, 0);
    must(world.insert(e, Foe, { slot: i }));
  }
  for (let i = 0; i < FLY_SLOTS; i++) {
    const e = must2(world.spawn({ name: `fly-${i}`, position: { x: 0, y: -40, z: 2 } }));
    must(world.insert(e, og.Sprite, { texture: cardArt[0] as never, width: 1.6, height: 1.28, layer: 6, opacity: 0 }));
  }

  const voices: [string, number, number][] = [
    ["audio/card.wav", Sound.Card, 0.45],
    ["audio/hit.wav", Sound.Hit, 0.6],
    ["audio/heavy.wav", Sound.Heavy, 0.7],
    ["audio/block.wav", Sound.Block, 0.5],
    ["audio/hurt.wav", Sound.Hurt, 0.6],
    ["audio/poison.wav", Sound.Poison, 0.5],
    ["audio/buff.wav", Sound.Buff, 0.5],
    ["audio/draw.wav", Sound.Draw, 0.4],
    ["audio/turn.wav", Sound.Turn, 0.5],
    ["audio/foe-turn.wav", Sound.FoeTurn, 0.5],
    ["audio/die.wav", Sound.Die, 0.6],
    ["audio/heal.wav", Sound.Heal, 0.5],
    ["audio/won.wav", Sound.Won, 0.7],
    ["audio/lost.wav", Sound.Lost, 0.7],
    ["audio/ui-press.ogt", Sound.Move, 0.3],
  ];
  for (const [path, bit, volume] of voices) {
    const v = must2(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: bit === Sound.Move ? "ui" : "sfx", volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must2(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/battle.wav"), bus: "music", loop: true, volume: 0.55, priority: 220 }));
  const fx = must2(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 5, duckRelease: 0.5 }));
  const room = must2(world.spawn({ name: "sfx-bus" }));
  must(world.insert(room, og.AudioBusFx, { bus: "sfx", reverb: 0.12 }));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

function seatArg(s: string | undefined): number {
  const seat = s === undefined ? 0 : num(s, "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seat is 0${SEATS > 1 ? ` to ${SEATS - 1}` : ""}`);
  return seat;
}

/** `begin`: start a run now, as Begin does. */
function begin(world: World): void {
  const c = load(world);
  if (!c) return;
  startRun(c);
  save(c);
}

/** `goto:<step>[:<path>]`: jump to a step of the map and take it (0 the first fight, 5 the King). */
function goto(world: World, args: readonly string[]): void {
  const c = load(world);
  if (!c) return;
  if (c.g.phase === Phase.Title || c.g.phase === Phase.Over) startRun(c);
  const step = num(args[0], "step");
  if (!Number.isInteger(step) || step < 0 || step >= MAP.length) throw new Error(`step is 0 to ${MAP.length - 1}`);
  c.g.step = step;
  enter(c, args[1] === undefined ? 0 : num(args[1], "path"));
  save(c);
}

/** `hp:<n>[:<seat>]`, `foehp:<slot>:<n>`, `energy:<n>[:<seat>]`: set a number. */
function setNumber(world: World, which: "hp" | "energy", args: readonly string[]): void {
  const c = load(world);
  if (!c) return;
  const h = c.heroes[seatArg(args[1])]!;
  h.v[which] = num(args[0], "n");
  save(c);
}

function foeHp(world: World, args: readonly string[]): void {
  const c = load(world);
  if (!c) return;
  const f = c.foes[num(args[0], "slot")];
  if (!f) throw new Error(`slot is 0 to ${FOE_SLOTS - 1}`);
  f.v.hp = Math.max(1, num(args[1], "n"));
  save(c);
}

/** `give:<card>[:<seat>]`: a card into a hero's hand now (and its deck). */
function give(world: World, args: readonly string[]): void {
  const c = load(world);
  if (!c) return;
  const code = CARD[(args[0] ?? "").replace(/\+$/, "")];
  if (code === undefined) throw new Error(`write give:<${CARDS.map((d) => d.id).join("|")}>[+]`);
  const h = c.heroes[seatArg(args[1])]!;
  h.v.deck = [...h.v.deck, code | ((args[0] ?? "").endsWith("+") ? UPGRADED : 0)];
  if (c.g.phase === Phase.Battle) h.v.hand = [...h.v.hand, h.v.deck.length - 1];
  save(c);
}

/** `play:<hand position>[:<foe slot>[:<seat>]]`: play a card of a hand as a tap would (no seat input: for scripts only). */
function play(world: World, args: readonly string[]): void {
  const c = load(world);
  if (!c) return;
  const h = c.heroes[seatArg(args[2])]!;
  const at = h.v.hand[num(args[0], "position")];
  if (at === undefined) throw new Error("no card there");
  playCard(c, h, at, args[1] === undefined ? -1 : num(args[1], "slot"));
  save(c);
}

function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  setLanguage(world, at);
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Seats, Ui, Run, Stage, Voices],
  render: { antialias: "off" },
  start,
  commands: {
    begin,
    goto,
    hp: (w, a) => setNumber(w, "hp", a),
    energy: (w, a) => setNumber(w, "energy", a),
    foehp: foeHp,
    give,
    play,
    viewport,
    language,
    bot: (_w, a) => setBot(Number(a[0])),
  },
  commandHelp: {
    begin: { usage: "", doc: "Start a run now, as Begin does." },
    goto: { usage: "<step>[:<path>]", doc: "Jump to a step of the map and take it (0 the first fight, 2:0 the fire, 5 the Ashen King)." },
    hp: { usage: "<n>[:<seat>]", doc: "Set a hero's health." },
    energy: { usage: "<n>[:<seat>]", doc: "Set a hero's energy." },
    foehp: { usage: "<slot>:<n>", doc: "Set a foe's health (at least 1)." },
    give: { usage: "<card>[+][:<seat>]", doc: "Put a card (upgraded with +) in a hero's deck, and in its hand in a fight." },
    play: { usage: "<position>[:<slot>[:<seat>]]", doc: "Play the card at a position of a hand on a foe slot (scripts only: a room's peers would not see it)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
  },
  facts: (world) => {
    const c = load(world);
    if (!c) return null;
    const g = c.g;
    return {
      phase: PHASES[g.phase],
      view: TOP ? "top-down-2d" : "side-2d",
      seats: SEATS,
      party: g.party,
      step: g.step,
      turn: g.turn,
      side: g.side === 0 ? "heroes" : "foes",
      heroes: c.heroes.filter((h) => h.v.active).map((h) => ({ hero: HEROES[h.v.seat], hp: h.v.hp, block: h.v.block, energy: h.v.energy, hand: h.v.hand.map((i) => CARDS[h.v.deck[i]! & 255]!.id), draw: h.v.draw.length, discard: h.v.discard.length, deck: h.v.deck.length, ended: h.v.ended })),
      foes: c.foes.filter((f) => f.v.active).map((f) => ({ foe: FOES[f.v.kind]!.id, hp: f.v.hp, block: f.v.block, poison: f.v.poison, vulnerable: f.v.vulnerable, intent: f.v.move })),
      down: partyDown(c),
      won: g.won,
      fights: g.fights,
      played: g.played,
      runs: g.runs,
      wins: g.wins,
      language: LANGUAGES[prefs.language]!.code,
    };
  },
});
