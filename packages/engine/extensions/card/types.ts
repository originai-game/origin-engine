// The card kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Rarity, Texture, UiRect } from "@origingame/engine";

/** A targeted card being aimed: what to preview (the damage on the enemy under it). */
export interface CardAim {
  /** The card. */
  readonly key: string;
  /** The valid or invalid target under the arc's tip; null over none. */
  readonly target: string | null;
  /**
   * The arc's tip.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * The arc's tip.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
}

/**
 * A hand of cards, its targets and what it says.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CardHandDesc {
  /**
   * The cards, first to last (start to end of the fan).
   *
   * @default []
   */
  readonly cards?: readonly HandCard[];
  /**
   * Where a targeted card can go: aiming snaps the arc to the one under the pointer, and keys and pads step through the valid ones.
   *
   * @default []
   */
  readonly targets?: readonly CardTarget[];
  /**
   * A card's width at rest; 0 is 150 on a desktop (scaled with the screen) and 104 on a phone. Its height is 1.4 times it.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cardWidth?: number;
  /**
   * How far the fan turns its outermost cards, in degrees; 0 is 3 a card, at most 18.
   *
   * @default 0
   * @range 0..40
   */
  readonly spread?: number;
  /**
   * How far up the screen a card must be dragged to play, as a fraction of the screen's height from its bottom; 0 is 0.36.
   *
   * @default 0
   * @range 0..1
   */
  readonly playLine?: number;
  /**
   * Not the player's turn: cards lift and can be read, none plays.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * The key of a card shown large in the middle (the details action, a long press); empty shows none.
   *
   * @default ""
   */
  readonly inspect?: string;
  /**
   * What a screen reader calls the hand.
   *
   * @default "Hand"
   */
  readonly label?: string;
  /**
   * What the arc's tip says while aiming with keys and pads.
   *
   * @default "Choose a target"
   */
  readonly targetPrompt?: string;
}

/** What the hand did this tick. */
export interface CardHandResult {
  /**
   * A card played: a non-targeted one released above the play line (or confirmed with keys), a targeted one released on a valid target (or confirmed on one).
   */
  readonly played: CardPlay | null;
  /** A targeted card being aimed. */
  readonly aiming: CardAim | null;
  /** The card under the pointer or the focus. */
  readonly hovered: string | null;
  /**
   * A card whose details were asked for: a long press, the details action; set CardHandDesc.inspect to it, and clear it on dismissed.
   */
  readonly inspect: string | null;
  /** The large card was dismissed (Back, a press). */
  readonly dismissed: boolean;
  /** An unplayable card the player tried to play: it shook, and the game may say why. */
  readonly refused: string | null;
}

/** A card played: the game spends its cost and resolves it. */
export interface CardPlay {
  /** The card. */
  readonly key: string;
  /** The target's key, for a card that needs one. */
  readonly target: string | null;
}

/**
 * A place a targeted card can be played on: an enemy, an ally, a lane, as the game projects it onto the screen.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CardTarget {
  /**
   * Its key, answered with the play.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What a screen reader calls it.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Where it is on the screen (og.ui.worldAnchor's or a HUD element's rectangle).
   *
   * @default { x: 0, y: 0, width: 0, height: 0 }
   */
  readonly rect?: UiRect;
  /**
   * The aimed card can be played on it: the arc ends in success ink and a ring marks it; an invalid one turns the arc danger.
   *
   * @default true
   */
  readonly valid?: boolean;
}

/**
 * One card of the collection, as the game holds it: the face og.ui.cardHand draws, the copies owned and in the deck.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CollectionCard {
  /**
   * Its key: its tile's element id (<id>/Collection/Grid/Line#<n>/<key>) and its deck row's (<id>/Deck/List/<key>).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, on its banner and its deck row.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its cost, the gem; null has none. The curve counts it (7 and up in the last bar).
   *
   * @default null
   */
  readonly cost?: number | null;
  /**
   * Its type line on the face (Spell, Unit).
   *
   * @default ""
   */
  readonly typeLine?: string;
  /**
   * Its kind's key, for the kind filter and the deck's counts (unit, spell).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * Its rules (rich text), shown on the large face of the card hovered or focused.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * Its art (art kind cardArt); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: the frame's colour, the rarity filter and the deck row's mark.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Copies owned: the tile's ×N; none dims the tile and it cannot be added.
   *
   * @default 0
   */
  readonly owned?: number;
  /**
   * Copies in the deck: its deck row's count.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * Copies a deck may hold; 0 is the deck's copyLimit.
   *
   * @default 0
   */
  readonly limit?: number;
}

/**
 * A deck builder over the collection: the filters, the collection's grid, the deck's list, its curve and counts, Done.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DeckBuilderDesc {
  /**
   * The screen's title (Collection).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Every card of the collection, in the game's order; the filters and the sort are applied here.
   *
   * @default []
   */
  readonly cards?: readonly CollectionCard[];
  /**
   * The deck's name, at the head of its list.
   *
   * @default ""
   */
  readonly deckName?: string;
  /**
   * The deck's art behind its name (art kind cardArt); null shows none.
   *
   * @default null
   */
  readonly deckImage?: Texture | null;
  /**
   * The cards a deck holds: the count's whole (30/30), and the limit a card past it is refused at.
   *
   * @default 30
   */
  readonly deckSize?: number;
  /**
   * Copies of one card a deck may hold, unless the card says otherwise.
   *
   * @default 2
   */
  readonly copyLimit?: number;
  /**
   * The cost shown, the last chip (7) meaning that and up; -1 all.
   *
   * @default -1
   */
  readonly costFilter?: number;
  /**
   * The tier shown; none all.
   *
   * @default "none"
   */
  readonly rarityFilter?: Rarity;
  /**
   * The kind shown; empty all.
   *
   * @default ""
   */
  readonly kindFilter?: string;
  /**
   * The search's text: names that hold it, whatever the case.
   *
   * @default ""
   */
  readonly search?: string;
  /**
   * The sort's key; empty keeps the game's order.
   *
   * @default ""
   */
  readonly sort?: string;
  /**
   * The tiers the rarity filter offers.
   *
   * @default []
   */
  readonly rarities?: readonly DeckRarity[];
  /**
   * The kinds: the filter's chips and the deck's counts.
   *
   * @default []
   */
  readonly kinds?: readonly DeckKind[];
  /**
   * The orders the sort offers.
   *
   * @default []
   */
  readonly sorts?: readonly DeckSort[];
  /**
   * On a phone in portrait, the half shown: `deck` the deck, else the collection.
   *
   * @default ""
   */
  readonly view?: string;
  /**
   * The filters' all.
   *
   * @default "All"
   */
  readonly allLabel?: string;
  /**
   * The search's placeholder.
   *
   * @default "Search cards"
   */
  readonly searchLabel?: string;
  /**
   * The sort's name.
   *
   * @default "Sort"
   */
  readonly sortLabel?: string;
  /**
   * The collection's tab on a phone, and its count's word ({owned} / {total} cards).
   *
   * @default "Cards"
   */
  readonly collectionLabel?: string;
  /**
   * The deck's heading (My deck).
   *
   * @default "My deck"
   */
  readonly deckLabel?: string;
  /**
   * The curve's heading.
   *
   * @default "Mana curve"
   */
  readonly curveLabel?: string;
  /**
   * The counts' heading.
   *
   * @default "Types"
   */
  readonly kindsLabel?: string;
  /**
   * Done.
   *
   * @default "Done"
   */
  readonly doneLabel?: string;
  /**
   * Said when a card is refused for a full deck.
   *
   * @default "The deck is full"
   */
  readonly fullLabel?: string;
  /**
   * Said when a card is refused at its copies' limit.
   *
   * @default "No more copies allowed"
   */
  readonly limitLabel?: string;
  /**
   * Said for a card not owned.
   *
   * @default "Not owned"
   */
  readonly unownedLabel?: string;
  /**
   * The grid's word when no card passes the filters.
   *
   * @default "No cards match"
   */
  readonly emptyLabel?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface DeckBuilderResult {
  /** A collection card was pressed and the deck can take it: add a copy. */
  readonly add: string | null;
  /** A deck row was pressed: take a copy out. */
  readonly remove: string | null;
  /**
   * A card was pressed that the deck cannot take (full, at its limit, not owned): it shakes and the reason shows.
   */
  readonly refused: string | null;
  /** A cost chip was pressed (-1 all). */
  readonly costFilter: number | null;
  /** A tier chip was pressed (none all). */
  readonly rarityFilter: Rarity | null;
  /** A kind chip was pressed (empty all). */
  readonly kindFilter: string | null;
  /** The search's text changed. */
  readonly search: string | null;
  /** Another order was chosen. */
  readonly sort: string | null;
  /** On a phone, the other half's tab was pressed. */
  readonly view: string | null;
  /** Done was pressed. */
  readonly done: boolean;
}

/**
 * A card kind: a filter chip and a line of the deck's counts.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DeckKind {
  /**
   * Its key, as the cards' kind.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its word (Units).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its mark (art kind icon); null shows the word alone.
   *
   * @default null
   */
  readonly image?: Texture | null;
}

/**
 * A tier the rarity filter offers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DeckRarity {
  /**
   * The tier.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Its word (Rare).
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * An order the collection can be sorted in: `cost`, `name`, `rarity` and `owned` are known; another key keeps the game's order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DeckSort {
  /**
   * Its key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its word.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A card in the player's hand, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface HandCard {
  /**
   * Its key, stable while it is in the hand: its element id, so its probe path is <id>/Fan/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, on its banner.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * What it costs to play, in its gem in the top start corner; null draws no gem (a free card, a land).
   *
   * @default null
   */
  readonly cost?: number | null;
  /**
   * The cost is raised this turn: the gem's number in danger ink.
   *
   * @default false
   */
  readonly costUp?: boolean;
  /**
   * The cost is lowered: the number in success ink.
   *
   * @default false
   */
  readonly costDown?: boolean;
  /**
   * Its type (Attack, Skill · Fire): a line under the art.
   *
   * @default ""
   */
  readonly typeLine?: string;
  /**
   * Its rules as rich-text markup (og.ui.richText): [b]Deal 6[/b] damage. [icon=…] and [color=…] are honoured.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * Its art (art kind card); without it a placeholder in its tier's colour.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: its frame's colour and pips.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * The player can play it now (enough energy, a legal moment): unplayable cards are dimmed, their cost in danger ink, and a drag of one shakes and is refused.
   *
   * @default false
   */
  readonly playable?: boolean;
  /**
   * Playing it asks for a target: dragged up, it stays over the hand and an arc follows the pointer to the target under it.
   *
   * @default false
   */
  readonly needsTarget?: boolean;
  /**
   * It is worth playing now (a combo, a synergy): its edge glows in the primary ink.
   *
   * @default false
   */
  readonly glow?: boolean;
  /**
   * Drawn this tick: it flies in from the draw pile.
   *
   * @default false
   */
  readonly fresh?: boolean;
}

/**
 * A choice of rewards and what it allows.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RewardChoiceDesc {
  /**
   * The rewards, start to end.
   *
   * @default []
   */
  readonly options?: readonly RewardOption[];
  /**
   * The heading over them (Choose a reward); empty is none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it (Pick 1 of 3).
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * How many to take: 1 takes the one pressed at once (or after confirm with confirm); more need confirm.
   *
   * @default 1
   * @range 1..
   */
  readonly picks?: number;
  /**
   * A press selects; a Confirm button takes the selection (a choice the player should not take by a slip).
   *
   * @default false
   */
  readonly confirm?: boolean;
  /**
   * The keys selected, for picks over one or confirm: the game passes back what it was answered.
   *
   * @default []
   */
  readonly selected?: readonly string[];
  /**
   * Rerolls left: a Reroll button with the count; 0 shows it spent; -1 shows none.
   *
   * @default -1
   */
  readonly rerolls?: number;
  /**
   * What a reroll costs, beside its label (50 gold); empty is free.
   *
   * @default ""
   */
  readonly rerollCost?: string;
  /**
   * Banishes left: a Banish button that arms banishing, after which a press on a reward banishes it; -1 shows none.
   *
   * @default -1
   */
  readonly banishes?: number;
  /**
   * A Skip button's label (Skip, Take 25 gold); empty shows none.
   *
   * @default ""
   */
  readonly skipLabel?: string;
  /**
   * The Confirm button's label.
   *
   * @default "Confirm"
   */
  readonly confirmLabel?: string;
  /**
   * The Reroll button's label.
   *
   * @default "Reroll"
   */
  readonly rerollLabel?: string;
  /**
   * The Banish button's label.
   *
   * @default "Banish"
   */
  readonly banishLabel?: string;
  /**
   * A reward's width; 0 is 260 on a desktop (scaled with the screen) and the phone's room. Its height follows its content.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cardWidth?: number;
}

/** What the choice did this tick. */
export interface RewardChoiceResult {
  /**
   * The rewards taken: one pressed (picks 1, no confirm), or the selection confirmed. Empty while none.
   */
  readonly picked: readonly string[];
  /** The selection after this tick's press (pass it back as RewardChoiceDesc.selected). */
  readonly selected: readonly string[];
  /** Reroll was pressed: offer new rewards and one fewer reroll. */
  readonly rerolled: boolean;
  /** A reward banished: remove it and one banish. */
  readonly banished: string | null;
  /** Skip was pressed. */
  readonly skipped: boolean;
  /** The reward under the pointer or the focus. */
  readonly hovered: string | null;
  /** A locked reward pressed: it shook. */
  readonly refused: string | null;
}

/**
 * One reward of a choice, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RewardOption {
  /**
   * Its key, stable while offered: its element id, so its probe path is <id>/Options/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What it is (Thunder Strike, +20 % Max Health).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its kind or source over the title (Relic, Gift of the Storm Warden, Weapon).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * What it does, as rich-text markup (og.ui.richText): [b], [color=…] and [icon=…] are honoured.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * Its art (art kind item or card); without it a placeholder in its tier's colour.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * An icon of the style's set for the placeholder.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its tier: the frame's colour, a glow from epic, and its pips.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * A short tag in its corner (New, Upgrade, Lv 3 → 4); empty draws none.
   *
   * @default ""
   */
  readonly tag?: string;
  /**
   * What the tag says, which sets its ink: new in the tier's colour, upgrade in success ink with an up arrow.
   *
   * @default "note"
   */
  readonly tagKind?: RewardTagKind;
  /**
   * It cannot be taken (a requirement unmet): shaded with a lock, and a press shakes.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Why, said under its text and to a screen reader.
   *
   * @default ""
   */
  readonly lockedHint?: string;
}

/**
 * What a reward's corner tag says.
 * - `"note"`: A plain note (Lv 3 → 4): the highlight.
 * - `"new"`: Never taken before: the tier's own ink, as a pull's New.
 * - `"upgrade"`: It improves one the player holds: the success ink with an up arrow.
 */
export type RewardTagKind = "note" | "new" | "upgrade";
