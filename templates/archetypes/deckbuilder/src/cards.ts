// Ashen Crown's cards: plain data, one row per card. A card in a deck is a
// code (its index in CARDS, plus UPGRADED when tempered at a fire); its
// words are string keys (`card.<id>`, `card.<id>.text` with the numbers as
// {a} and {b}), its art assets/cards/<id>.png. src/battle.ts resolves them.

/** What a card is: its banner's type line and its frame's tier. */
export type CardKind = "attack" | "skill" | "power";

export interface CardDef {
  readonly id: string;
  readonly kind: CardKind;
  /** Its cost in energy, then upgraded. */
  readonly cost: readonly [number, number];
  /** It is played on one foe. */
  readonly target: boolean;
  /** Its numbers ({a}, {b} in its text), then upgraded. */
  readonly a: readonly [number, number];
  readonly b: readonly [number, number];
  /** Played, it leaves the fight (it goes to no pile). */
  readonly exhaust?: boolean;
  readonly rarity: "common" | "uncommon" | "rare";
}

/** Every card, in a fixed order: a deck holds indices into it. */
export const CARDS: readonly CardDef[] = [
  { id: "strike", kind: "attack", cost: [1, 1], target: true, a: [6, 9], b: [0, 0], rarity: "common" },
  { id: "defend", kind: "skill", cost: [1, 1], target: false, a: [5, 8], b: [0, 0], rarity: "common" },
  { id: "bash", kind: "attack", cost: [2, 2], target: true, a: [8, 10], b: [2, 3], rarity: "common" },
  { id: "cleave", kind: "attack", cost: [1, 1], target: false, a: [8, 11], b: [0, 0], rarity: "common" },
  { id: "venom-dart", kind: "attack", cost: [1, 1], target: true, a: [3, 5], b: [4, 6], rarity: "common" },
  { id: "toxic-cloud", kind: "skill", cost: [2, 2], target: false, a: [6, 9], b: [0, 0], rarity: "uncommon" },
  { id: "ward", kind: "skill", cost: [2, 2], target: false, a: [10, 14], b: [5, 7], rarity: "uncommon" },
  { id: "insight", kind: "skill", cost: [0, 0], target: false, a: [2, 3], b: [0, 0], rarity: "common" },
  { id: "adrenaline", kind: "skill", cost: [0, 0], target: false, a: [1, 2], b: [2, 2], exhaust: true, rarity: "rare" },
  { id: "flurry", kind: "attack", cost: [1, 1], target: true, a: [3, 3], b: [3, 4], rarity: "uncommon" },
  { id: "iron-hymn", kind: "power", cost: [1, 1], target: false, a: [2, 3], b: [0, 0], rarity: "uncommon" },
  { id: "heavy-blow", kind: "attack", cost: [2, 2], target: true, a: [14, 19], b: [0, 0], rarity: "common" },
  { id: "rally", kind: "skill", cost: [1, 0], target: false, a: [2, 2], b: [1, 1], rarity: "uncommon" },
  { id: "sunder", kind: "attack", cost: [1, 1], target: true, a: [7, 9], b: [2, 3], rarity: "common" },
  { id: "kindle", kind: "power", cost: [1, 0], target: false, a: [1, 1], b: [0, 0], rarity: "rare" },
];

export const CARD = Object.fromEntries(CARDS.map((c, i) => [c.id, i])) as Record<string, number>;

/** A card code's upgraded bit. */
export const UPGRADED = 256;

export function defOf(code: number): CardDef {
  const d = CARDS[code & 255];
  if (!d) throw new Error(`no card ${code & 255}`);
  return d;
}

export function isUpgraded(code: number): boolean {
  return (code & UPGRADED) !== 0;
}

export function costOf(code: number): number {
  return defOf(code).cost[isUpgraded(code) ? 1 : 0];
}

export function numbers(code: number): { a: number; b: number } {
  const d = defOf(code);
  const u = isUpgraded(code) ? 1 : 0;
  return { a: d.a[u], b: d.b[u] };
}

/** Each seat's starting deck: ten cards. */
export const STARTER_DECKS: readonly (readonly number[])[] = [
  // The Warden: plate and a shield.
  [CARD.strike!, CARD.strike!, CARD.strike!, CARD.strike!, CARD.defend!, CARD.defend!, CARD.defend!, CARD.defend!, CARD.bash!, CARD.ward!],
  // The Hunter: venom and quick hands.
  [CARD.strike!, CARD.strike!, CARD.strike!, CARD.defend!, CARD.defend!, CARD.defend!, CARD.defend!, CARD["venom-dart"]!, CARD["venom-dart"]!, CARD.insight!],
];

/** What a fight's reward draws from: every card but the two basics. */
export const REWARD_POOL: readonly number[] = CARDS.map((_, i) => i).filter((i) => CARDS[i]!.id !== "strike" && CARDS[i]!.id !== "defend");

/** The most cards a deck, a pile or a hand holds. */
export const DECK_MAX = 40;
export const HAND_MAX = 10;
