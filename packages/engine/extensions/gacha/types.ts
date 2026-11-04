// The gacha kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Rarity, Texture, Vector2 } from "@origingame/engine";

/**
 * A gacha banner (its art, its featured units, its rules) and the player's purse.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GachaBannerDesc {
  /**
   * The banners; one or none draws no switcher.
   *
   * @default []
   */
  readonly banners?: readonly GachaBannerTab[];
  /**
   * The key of the banner shown.
   *
   * @default ""
   */
  readonly current?: string;
  /**
   * The banner's title over the art.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it (Limited banner · Character).
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * The key art (art kind banner), across the frame around its focal point under the art builder's layer; null draws the style's placeholder: a figure on a gradient of two hues, lit at the focal point.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * The art's point kept in view, fractions from the top start; null is the featured unit's place on the art side (0.68, 0.36 on a desktop; 0.5, 0.3 in portrait).
   *
   * @default null
   */
  readonly focal?: Vector2 | null;
  /**
   * Room kept clear over the words and tabs for the game's own chrome on the art (a back button, a menu): the art and a short top shade run under it. Laid out to the screen's edges, the banner also keeps its words and controls inside the safe area while its art fills the notch and status bar.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly topInset?: number;
  /**
   * The featured units: the first named large on the art (its name plate) and each in a row with its stars beside the title.
   *
   * @default []
   */
  readonly featured?: readonly GachaFeatured[];
  /**
   * The rate-up rule, as rich text ([b]50 %[/b] chance the 5★ is featured).
   *
   * @default ""
   */
  readonly rateText?: string;
  /**
   * Seconds until it ends: a timer chip, urgent in its last hour.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly endsIn?: number | null;
  /**
   * Pulls made toward the guarantee.
   *
   * @default 0
   */
  readonly pity?: number;
  /**
   * Pulls at which a top tier is guaranteed; 0 shows no pity meter.
   *
   * @default 0
   */
  readonly pityMax?: number;
  /**
   * What the meter says ({left} is the pulls left).
   *
   * @default "Guaranteed in {left}"
   */
  readonly pityLabel?: string;
  /**
   * The pull currency's name (its icon when the style has one).
   *
   * @default "gem"
   */
  readonly currency?: string;
  /**
   * The currency's own art, drawn wherever a price or the purse names it; null draws the style's icon of that name, else a gem drawn as a shape.
   *
   * @default null
   */
  readonly currencyIcon?: Texture | null;
  /**
   * How much of it the player has.
   *
   * @default 0
   */
  readonly balance?: number;
  /**
   * What one pull costs.
   *
   * @default 0
   */
  readonly single?: number;
  /**
   * What ten cost; 0 hides the ten-pull.
   *
   * @default 0
   */
  readonly multi?: number;
  /**
   * How many the multi pull draws.
   *
   * @default 10
   */
  readonly multiCount?: number;
  /**
   * The one-pull button.
   *
   * @default "Summon ×1"
   */
  readonly singleLabel?: string;
  /**
   * The ten-pull button ({n} is multiCount).
   *
   * @default "Summon ×{n}"
   */
  readonly multiLabel?: string;
  /**
   * Said with a price the balance cannot pay.
   *
   * @default "not enough"
   */
  readonly shortLabel?: string;
  /**
   * The rates and details button.
   *
   * @default "Details"
   */
  readonly detailsLabel?: string;
  /**
   * The pull history button.
   *
   * @default "History"
   */
  readonly historyLabel?: string;
  /**
   * A time left of a day or more, on the timer and the tabs ({d} days, {h} hours).
   *
   * @default "{d}d {h}h"
   */
  readonly daysLabel?: string;
  /**
   * A time left of an hour or more ({h} hours, {m} minutes).
   *
   * @default "{h}h {m}m"
   */
  readonly hoursLabel?: string;
  /**
   * A time left under an hour, on the tabs ({m} minutes; the timer counts it down as m:ss).
   *
   * @default "{m}m"
   */
  readonly minutesLabel?: string;
}

/** What the banner did this tick. */
export interface GachaBannerResult {
  /**
   * Pulls asked for this tick: 1, multiCount, or 0. The game checks the purse, spends it and starts gacha.pullSequence.
   */
  readonly pull: number;
  /** Pulls asked for that the balance cannot pay: offer the top-up (0 when none). */
  readonly short: number;
  /** Another banner picked. */
  readonly banner: string | null;
  /** Details was pressed. */
  readonly details: boolean;
  /** History was pressed. */
  readonly history: boolean;
}

/**
 * One banner of the set the player switches between.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GachaBannerTab {
  /**
   * Its key: its element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its short name (Event, Weapon, Standard).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A thumbnail (art kind banner); without it the tier wash.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Seconds until it ends; null for a permanent banner.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly endsIn?: number | null;
  /**
   * New since the player last looked: a dot.
   *
   * @default false
   */
  readonly fresh?: boolean;
}

/**
 * A featured unit of a banner (its rate-up).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GachaFeatured {
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A line under it (Lightning Swordswoman).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its stars, drawn as stars (never a glyph a font may lack).
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Its tier's colour.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Its portrait (art kind portrait), in its tile; without it the style's placeholder figure.
   *
   * @default null
   */
  readonly image?: Texture | null;
}

/**
 * One result of a pull.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PullItem {
  /**
   * Its key within the pull: its element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What was drawn.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * What sort it is (Character, Weapon, Light Cone).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * Its stars, drawn as stars.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Its tier: the reveal's colour (the highest tier of a pull tints its charge), its glow, its burst from legendary.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Its art (art kind character or item).
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Drawn for the first time: a New tag.
   *
   * @default false
   */
  readonly isNew?: boolean;
  /**
   * What a duplicate turned into (+10 Star Shards); empty for none.
   *
   * @default ""
   */
  readonly extra?: string;
  /**
   * Its tier's word in the player's language (Legendary), set in the tier's ink before its kind on the reveal, so the tier is read in a word as well as in colour and stars; empty takes the tier's English name.
   *
   * @default ""
   */
  readonly rarityLabel?: string;
  /**
   * A character's epithet, a caption in its tier's ink under the name (the banner's grammar: the name large, the epithet under it); empty for none.
   *
   * @default ""
   */
  readonly title?: string;
}

/**
 * Where a pull sequence is.
 * - `"idle"`: Nothing shown (token 0, or closed).
 * - `"charge"`: The charge: a light gathering in the colour of the pull's highest tier.
 * - `"reveal"`: One result shown large, one after another.
 * - `"summary"`: Every result at once.
 */
export type PullPhase = "idle" | "charge" | "reveal" | "summary";

/**
 * A pull to reveal: start one by giving a new token.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PullSequenceDesc {
  /**
   * The pull's number: a change starts the sequence again from its charge; 0 shows nothing.
   *
   * @default 0
   */
  readonly token?: number;
  /**
   * What was drawn, in reveal order.
   *
   * @default []
   */
  readonly items?: readonly PullItem[];
  /**
   * The pull-again button in the summary ({n} is the count); empty shows none.
   *
   * @default "Again ×{n}"
   */
  readonly againLabel?: string;
  /**
   * Its price, in currency.
   *
   * @default 0
   */
  readonly againPrice?: number;
  /**
   * The currency of againPrice.
   *
   * @default "gem"
   */
  readonly currency?: string;
  /**
   * The currency's own art beside againPrice; null draws the style's icon, else a gem.
   *
   * @default null
   */
  readonly currencyIcon?: Texture | null;
  /**
   * The banner's scene behind the charge and the reveal, blurred and dimmed under the tier's light (the banner's key art); null is the stage's dark.
   *
   * @default null
   */
  readonly backdrop?: Texture | null;
  /**
   * The purse pays againPrice.
   *
   * @default true
   */
  readonly affordable?: boolean;
  /**
   * The summary's close button.
   *
   * @default "Done"
   */
  readonly doneLabel?: string;
  /**
   * The skip-to-summary button.
   *
   * @default "Skip"
   */
  readonly skipLabel?: string;
  /**
   * What the reveal says to go on on a touch screen.
   *
   * @default "Tap to continue"
   */
  readonly tapLabel?: string;
  /**
   * What it says after the confirm glyph when a key or a pad was used last (one hint for the device in hand, never both).
   *
   * @default "Continue"
   */
  readonly continueLabel?: string;
  /**
   * The badge on a unit seen for the first time.
   *
   * @default "NEW"
   */
  readonly newLabel?: string;
}

/** What the sequence did this tick. */
export interface PullSequenceResult {
  /** Where it is. */
  readonly phase: PullPhase;
  /** The result shown in reveal (0 first). */
  readonly index: number;
  /** A result revealed this tick (play its voice line, count it seen). */
  readonly revealed: string | null;
  /** Pull again was pressed in the summary. */
  readonly again: boolean;
  /** Done was pressed (or Back in the summary): it closed; set token to 0. */
  readonly done: boolean;
}
