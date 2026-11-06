// The idle kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Texture } from "@origingame/engine";

/**
 * A generator of an idle game, as the game's viewmodel holds it. The game writes its numbers (og.ui.formatNumber, idle letters), so they read in its notation and language.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IdleGeneratorDesc {
  /**
   * What it is (Solar Array).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its art (art kind item); null draws icon or a bolt.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * An icon of the style's set for it when it has no art.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its level.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * How many the player owns, as written (348); empty hides it.
   *
   * @default ""
   */
  readonly ownedText?: string;
  /**
   * What it makes, as written (+1.24 M / sec); empty hides it.
   *
   * @default ""
   */
  readonly rateText?: string;
  /**
   * How far its current cycle is, 0 to 1.
   *
   * @default 0
   * @range 0..1
   */
  readonly cycle?: number;
  /**
   * How long a cycle takes: under a quarter second the bar stays full and shimmers (too fast to read as a fill). 0 hides the bar.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly cycleSeconds?: number;
  /**
   * What the next purchase costs, as written (12.4 M).
   *
   * @default ""
   */
  readonly costText?: string;
  /**
   * The currency's art beside the cost; null draws a coin.
   *
   * @default null
   */
  readonly currencyIcon?: Texture | null;
  /**
   * The purse pays the purchase: the buy button lights.
   *
   * @default false
   */
  readonly affordable?: boolean;
  /**
   * The buy button's label ({n} is buyCount).
   *
   * @default "Buy ×{n}"
   */
  readonly buyLabel?: string;
  /**
   * How many a press buys (×1, ×10, the most the purse pays).
   *
   * @default 1
   */
  readonly buyCount?: number;
  /**
   * The level badge ({level}).
   *
   * @default "Lv. {level}"
   */
  readonly levelLabel?: string;
  /**
   * The owned line ({owned} is ownedText).
   *
   * @default "Owned: {owned}"
   */
  readonly ownedLabel?: string;
  /**
   * Not unlocked yet: dimmed with a lock and lockedHint in place of its numbers; it cannot be bought.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * What unlocks it (Reach 1 B energy).
   *
   * @default ""
   */
  readonly lockedHint?: string;
}

/** What the generator's row did this tick. */
export interface IdleGeneratorResult {
  /** Buy was pressed, or held long enough to repeat, and the purse pays: buy buyCount. */
  readonly bought: boolean;
  /** Buy was pressed without the purse to pay: say so (it shook). */
  readonly short: boolean;
  /** Buy is being held: it repeats every few ticks while the purse pays. */
  readonly held: boolean;
}

/**
 * An idle game's prestige: what a reset gives and what it costs.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PrestigeDesc {
  /**
   * The panel's heading (Prestige).
   *
   * @default "Prestige"
   */
  readonly title?: string;
  /**
   * The line over the reward (Ascend for).
   *
   * @default ""
   */
  readonly lead?: string;
  /**
   * What a reset gives, as written (+128 Shards): the panel's display line.
   *
   * @default ""
   */
  readonly rewardText?: string;
  /**
   * What a reset does, one line each with a tick (Resets generators, Permanent progression).
   *
   * @default []
   */
  readonly points?: readonly string[];
  /**
   * Its emblem (art kind emblem); null draws a crown.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * The hold-to-confirm button's label (Ascend).
   *
   * @default "Ascend"
   */
  readonly actionLabel?: string;
  /**
   * A reset gives something now: the button can be held.
   *
   * @default false
   */
  readonly available?: boolean;
  /**
   * Why not yet, under the button while unavailable (Reach 1 Qa energy).
   *
   * @default ""
   */
  readonly lockedHint?: string;
  /**
   * How long the button is held to confirm: a reset is not taken by a slip.
   *
   * @default 1200
   * @unit ms (milliseconds)
   */
  readonly holdMs?: number;
}

/** What the prestige panel did this tick. */
export interface PrestigeResult {
  /** The hold completed: reset the run and grant the reward. */
  readonly confirmed: boolean;
  /** The button is being held. */
  readonly holding: boolean;
}
