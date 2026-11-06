// Every piece of Ashen Crown's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The run (Game) and the knobs (Tuning) are resources. A hero per seat is
// an entity with its og.PlayerInput (src/seats.ts) and its Hero row: its
// health, block, energy, statuses and its cards (the deck it owns, and in
// a fight its draw pile, hand and discard pile as indices into that deck).
// The foes are three pooled Foe rows a fight switches on. Sounds are one
// Sfx row a sound. Card codes are src/cards.ts's.

import { bool, defineComponent, defineResource, f64, i32, list, u32 } from "@origingame/engine";
import type { Value } from "@origingame/engine";
import { DECK_MAX, HAND_MAX } from "./cards.ts";

/** The screen the run is on. */
export const Phase = { Title: 0, Map: 1, Battle: 2, Reward: 3, Rest: 4, Over: 5 } as const;
export const PHASES = ["title", "map", "battle", "reward", "rest", "over"] as const;

/** Whose side acts in a fight. */
export const Side = { Heroes: 0, Foes: 1 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Card: 1, Hit: 2, Heavy: 4, Block: 8, Hurt: 16, Poison: 32, Buff: 64, Draw: 128, Turn: 256, FoeTurn: 512, Die: 1024, Heal: 2048, Won: 4096, Lost: 8192, Move: 16384 } as const;

/**
 * What a seat's og.PlayerInput.aimX asks for with its play bit (aim is
 * `none`, so the game carries its own codes there; src/ui.ts's buttons and
 * cards set them, src/run.ts reads them): 0 the focused card on the focused
 * foe; a card is its deck index + 1 (aimY its foe slot + 1); the rest are
 * the menus' choices.
 */
export const Ask = { Focus: 0, Reward: 50, Skip: 59, Path: 60, Heal: 70, Temper: 71, TemperPick: 72, Begin: 80, Again: 81, Title: 82, Card: 1 } as const;

/** The run: its screen, its step on the map, the fight's turn, the rewards offered, the counts and the record. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  paused: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  seed: u32(0, { doc: "The run's seed (drawn from og.Rng when it starts): the shuffles and the rewards follow from og.Rng after it." }),
  party: u32(1, { doc: "Heroes in this run: the seats, or 1 online before the room runs." }),
  step: u32(0, { doc: "The map's step the party is on (src/foes.ts MAP)." }),
  path: u32(0, { doc: "The option taken at this step (a fork has two)." }),
  turn: u32(0, { doc: "The fight's turn, from 1." }),
  side: u32(0, { doc: "Side.*: whose turn it is." }),
  acting: i32(-1, { doc: "The foe slot acting now on the foes' turn (-1 none)." }),
  timer: u32(0, { unit: "ticks", doc: "Ticks to the next beat of the foes' turn, or of a fight's end." }),
  cleared: bool(false, { doc: "Every foe of the fight is down: the reward comes after a beat." }),
  rewards0: list(u32(), 3, { doc: "The cards seat 0 is offered after a fight." }),
  rewards1: list(u32(), 3),
  tempers0: list(u32(), 3, { doc: "Seat 0's deck indices a fire's Temper offers to upgrade (empty: not tempering)." }),
  tempers1: list(u32(), 3),
  decided: u32(0, { doc: "Seats that chose their reward or their rest (a bit each): the map comes once every hero has." }),
  won: bool(false),
  overTicks: u32(0, { unit: "ticks" }),
  fights: u32(0, { doc: "Fights won this run." }),
  played: u32(0, { doc: "Cards played this run." }),
  dealt: u32(0, { doc: "Damage dealt this run." }),
  taken: u32(0, { doc: "Damage taken this run." }),
  turns: u32(0, { doc: "Turns taken this run." }),
  killedBy: i32(-1, { doc: "The foe kind that struck the last hero down (-1 none)." }),
  runs: u32(0, { doc: "Runs begun on this device (the store's `record`)." }),
  wins: u32(0, { doc: "Runs won (the store's `record`)." }),
  recordRequest: u32(0),
  recordRevision: u32(0),
  recordWrite: u32(0),
  sfx: u32(0),
  banner: u32(0, { doc: "The turn's banner: 1 your turn, 2 the foes', since bannerAt." }),
  bannerAt: i32(-100, { unit: "ticks" }),
  flyNext: u32(0, { doc: "The next of the flying cards' pool (a played card arcs to its target)." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  heroHp: i32(60, { min: 1, max: 200, step: 1, tunable: true, label: "A hero's health" }),
  energy: i32(3, { min: 1, max: 9, step: 1, tunable: true, label: "Energy a turn" }),
  draw: i32(5, { min: 1, max: 10, step: 1, tunable: true, label: "Cards drawn a turn" }),
  restHeal: f64(0.3, { min: 0, max: 1, step: 0.05, tunable: true, label: "A fire heals", doc: "The fraction of a hero's health a rest gives back." }),
  foeBeat: u32(42, { min: 10, max: 120, unit: "ticks", tunable: true, label: "A foe's beat", doc: "Ticks between foes acting on their turn." }),
  hitStop: u32(4, { min: 0, max: 12, unit: "ticks", tunable: true, label: "Hit stop" }),
});

/** A hero (seat 0 the Warden, seat 1 the Hunter): its numbers, its cards, its focus by keys and pads. */
export const Hero = defineComponent("Hero", {
  seat: u32(0, { doc: "Its og.PlayerInput's player." }),
  active: bool(false, { doc: "In this run." }),
  hp: i32(0),
  hpMax: i32(0),
  block: i32(0),
  energy: i32(0),
  strength: i32(0),
  weak: i32(0, { doc: "Turns its attacks deal a quarter less." }),
  vulnerable: i32(0, { doc: "Turns it takes half again." }),
  kindle: i32(0, { doc: "Extra energy each turn (Kindle)." }),
  deck: list(u32(), DECK_MAX, { doc: "The cards it owns (codes of src/cards.ts)." }),
  draw: list(u32(), DECK_MAX, { doc: "The draw pile, as deck indices, its top last." }),
  hand: list(u32(), HAND_MAX, { doc: "The hand, as deck indices, left to right." }),
  discard: list(u32(), DECK_MAX, { doc: "The discard pile, as deck indices." }),
  fresh: list(u32(), HAND_MAX, { doc: "The deck indices drawn on freshAt (they fly in)." }),
  freshAt: i32(-10, { unit: "ticks" }),
  ended: bool(false, { doc: "It ended its turn: the foes act once every hero up has." }),
  focusCard: u32(0, { doc: "The hand's card the d-pad is on." }),
  focusFoe: u32(0, { doc: "The foe slot the d-pad is on." }),
  before: u32(0, { doc: "Its og.PlayerInput.buttons last tick (for presses)." }),
  lastDir: u32(0, { doc: "The d-pad's direction last tick (0 none, 1 up, 2 right, 3 down, 4 left)." }),
  refusedAt: i32(-100, { unit: "ticks", doc: "A card it could not play (too dear, no foe): the hand shakes it." }),
  refused: u32(0, { doc: "That card's deck index." }),
});

/** A foe: a pooled row a fight switches on. Its moves are src/foes.ts's. */
export const Foe = defineComponent("Foe", {
  slot: u32(0),
  active: bool(false),
  kind: u32(0),
  hp: i32(0),
  hpMax: i32(0),
  block: i32(0),
  strength: i32(0),
  weak: i32(0),
  vulnerable: i32(0),
  poison: i32(0, { doc: "Health it loses at the start of its turn (then one less)." }),
  move: u32(0, { doc: "Its next move: an index into its moves (its intent)." }),
  target: i32(0, { doc: "The seat its next move strikes (-1 every hero)." }),
  dying: u32(0, { unit: "ticks" }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});

export type GameV = Value<typeof Game>;
export type HeroV = Value<typeof Hero>;
export type FoeV = Value<typeof Foe>;

/** A seat's hero entity: `hero`, then `hero-1`. */
export function heroName(seat: number): string {
  return seat === 0 ? "hero" : `hero-${seat}`;
}

export function foeName(slot: number): string {
  return `foe-${slot}`;
}

export const HEROES = ["warden", "hunter"] as const;
