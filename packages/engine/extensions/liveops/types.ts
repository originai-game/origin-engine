// The liveops kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Rarity, Texture, Vector2 } from "@origingame/engine";

/**
 * A battle pass: the season, the player's level and XP, the premium track's ownership and the tiers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BattlePassDesc {
  /**
   * The season's name (Season 4 · Neon Tide).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The season's key art behind the header (art kind banner); null draws a gradient of the style's hues.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Seconds until the season ends: a timer chip in the header.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly endsIn?: number | null;
  /**
   * The player's level: tiers up to it can be claimed.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * XP toward the next level.
   *
   * @default 0
   */
  readonly xp?: number;
  /**
   * XP the next level takes.
   *
   * @default 0
   */
  readonly xpNext?: number;
  /**
   * The premium track is owned: its rewards can be claimed; until then they are locked.
   *
   * @default false
   */
  readonly premium?: boolean;
  /**
   * The tiers, first to last.
   *
   * @default []
   */
  readonly tiers?: readonly PassTier[];
  /**
   * The premium track's price on the unlock button ($9.99, or 950 with its currency); empty shows only the label.
   *
   * @default ""
   */
  readonly unlockPriceText?: string;
  /**
   * What a screen reader calls the pass.
   *
   * @default "Battle pass"
   */
  readonly label?: string;
  /**
   * The free track's name.
   *
   * @default "Free"
   */
  readonly freeLabel?: string;
  /**
   * The premium track's name.
   *
   * @default "Premium"
   */
  readonly premiumLabel?: string;
  /**
   * The player's level ({n}).
   *
   * @default "Level {n}"
   */
  readonly levelLabel?: string;
  /**
   * The XP toward the next level ({xp} and {need}).
   *
   * @default "{xp} / {need} XP"
   */
  readonly xpLabel?: string;
  /**
   * A claimable reward's word.
   *
   * @default "Claim"
   */
  readonly claimLabel?: string;
  /**
   * The button claiming every reward ready ({n} of them).
   *
   * @default "Claim all ({n})"
   */
  readonly claimAllLabel?: string;
  /**
   * What a claimed reward adds.
   *
   * @default "Claimed"
   */
  readonly claimedLabel?: string;
  /**
   * The button unlocking the premium track.
   *
   * @default "Unlock Premium"
   */
  readonly unlockLabel?: string;
  /**
   * What a premium reward adds while the track is locked.
   *
   * @default "Premium"
   */
  readonly lockedLabel?: string;
  /**
   * What a reward past the player's level adds ({n}).
   *
   * @default "Reach level {n}"
   */
  readonly reachLabel?: string;
  /**
   * A time left of a day or more ({d} days, {h} hours).
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
}

/** What the pass did this tick. */
export interface BattlePassResult {
  /**
   * Rewards claimed this tick (one pressed, or every one ready with Claim all): grant them and mark them claimed.
   */
  readonly claimed: readonly PassClaim[];
  /** Unlock Premium pressed: open its purchase. */
  readonly unlock: boolean;
  /**
   * A reward pressed that cannot be claimed yet (past the level, or premium while locked): it shook and said why.
   */
  readonly refused: PassClaim | null;
  /** The level of the tier under the pointer or the focus. */
  readonly hovered: number | null;
}

/** A reward claimed on a battle pass. */
export interface PassClaim {
  /** The tier's level. */
  readonly level: number;
  /** From the premium track. */
  readonly premium: boolean;
}

/**
 * One reward on a battle pass's track.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PassReward {
  /**
   * Its key: its element id within its tier.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What it is (Neon Outfit, 500 Coins).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A count on its corner (×500); empty is none.
   *
   * @default ""
   */
  readonly amountText?: string;
  /**
   * Its art (art kind item); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: the cell's rim and wash.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
}

/**
 * One level of a battle pass: its free and premium rewards and what the player has claimed.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PassTier {
  /**
   * Its level.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * The free track's reward; null is none at this level.
   *
   * @default null
   */
  readonly free?: PassReward | null;
  /**
   * The premium track's reward; null is none.
   *
   * @default null
   */
  readonly premium?: PassReward | null;
  /**
   * The free reward is claimed.
   *
   * @default false
   */
  readonly freeClaimed?: boolean;
  /**
   * The premium reward is claimed.
   *
   * @default false
   */
  readonly premiumClaimed?: boolean;
  /**
   * A milestone level (every tenth, the season's outfit): its cells larger, its level badge in gold.
   *
   * @default false
   */
  readonly milestone?: boolean;
}

/**
 * What a reward's corner tag says.
 * - `"note"`: A plain note (Lv 3 → 4): the highlight.
 * - `"new"`: Never taken before: the tier's own ink, as a pull's New.
 * - `"upgrade"`: It improves one the player holds: the success ink with an up arrow.
 */
export type RewardTagKind = "note" | "new" | "upgrade";

/**
 * A store front: the purse, then sections of offers on rotation.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StoreFrontDesc {
  /**
   * The store's title over the purse; empty shows none (the screen has its own header).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The sections, top to bottom.
   *
   * @default []
   */
  readonly sections?: readonly StoreSection[];
  /**
   * The purse at the head of the store, in order; empty shows none.
   *
   * @default []
   */
  readonly wallet?: readonly WalletEntry[];
  /**
   * The offer the game has open (its detail or confirm): drawn selected.
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * Tiles a row on a desktop; 0 fits as many 220 px tiles as the width takes (two on a phone).
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * What a screen reader calls the store.
   *
   * @default "Store"
   */
  readonly label?: string;
  /**
   * A free offer's price.
   *
   * @default "Free"
   */
  readonly freeLabel?: string;
  /**
   * An owned offer's line in place of its price.
   *
   * @default "Owned"
   */
  readonly ownedLabel?: string;
  /**
   * An offer bought up to its limit.
   *
   * @default "Sold out"
   */
  readonly soldOutLabel?: string;
  /**
   * What a limited offer has left ({left} of {limit}).
   *
   * @default "{left} left"
   */
  readonly leftLabel?: string;
  /**
   * A discount's ribbon ({pct} is the saving in whole percent).
   *
   * @default "−{pct} %"
   */
  readonly saveLabel?: string;
  /**
   * Beside a section's timer.
   *
   * @default "Refreshes in"
   */
  readonly refreshLabel?: string;
  /**
   * Said with a price the purse cannot pay.
   *
   * @default "not enough"
   */
  readonly shortLabel?: string;
  /**
   * A currency's plus to a screen reader ({name}).
   *
   * @default "Get more {name}"
   */
  readonly topUpLabel?: string;
  /**
   * A time left of a day or more ({d} days, {h} hours).
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
}

/** What the store did this tick. */
export interface StoreFrontResult {
  /**
   * An offer pressed that can be bought: open its detail or a confirm (og.ui.confirm), then charge it and pass bought or owned back.
   */
  readonly chosen: string | null;
  /** An owned offer pressed: show it (a preview, a fitting room). */
  readonly viewed: string | null;
  /** An offer pressed that cannot be bought (short or sold out): it shook and said why. */
  readonly refused: string | null;
  /** A currency's plus pressed: open the currency's own offers. */
  readonly topUp: string | null;
  /** The offer under the pointer or the focus. */
  readonly hovered: string | null;
}

/**
 * One offer of a store, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StoreOffer {
  /**
   * Its key, stable while it is offered: its element id, so its probe path is <id>/Sections/<section>/Offers/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What it is called.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A line under its name (Outfit, Bundle · 5 items).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * Its art covering the tile (art kind item, splash for a featured bundle); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * The art's point kept in view, fractions from the top start; null is its centre a little above the middle.
   *
   * @default null
   */
  readonly focal?: Vector2 | null;
  /**
   * Its tier: the tile's rim, a wash of its colour and its name's ink.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Its price in currency; 0 with the store's freeLabel is free.
   *
   * @default 0
   */
  readonly price?: number;
  /**
   * The purse's key it is priced in; empty with priceText is real money.
   *
   * @default ""
   */
  readonly currency?: string;
  /**
   * The price as written for real money ($4.99, from the platform's store); empty writes price with the currency's icon.
   *
   * @default ""
   */
  readonly priceText?: string;
  /**
   * The price before a discount: struck through beside it, the saving on a ribbon.
   *
   * @default 0
   */
  readonly original?: number;
  /**
   * What a bundle holds, a line each, listed on a featured tile.
   *
   * @default []
   */
  readonly contents?: readonly string[];
  /**
   * A corner tag (New, Hot, Limited); empty is none.
   *
   * @default ""
   */
  readonly badge?: string;
  /**
   * The tag's ink.
   *
   * @default "note"
   */
  readonly badgeKind?: RewardTagKind;
  /**
   * How many times it can be bought; 0 is no limit. A limit shows what is left, and none left is sold out.
   *
   * @default 0
   */
  readonly limit?: number;
  /**
   * How many times the player has bought it.
   *
   * @default 0
   */
  readonly bought?: number;
  /**
   * Owned already (a cosmetic): a tick and the ownedLabel instead of the price; a press answers viewed.
   *
   * @default false
   */
  readonly owned?: boolean;
  /**
   * The purse pays it: false draws the price in the danger ink, said short, and a press shakes and is refused.
   *
   * @default true
   */
  readonly affordable?: boolean;
  /**
   * A featured tile: twice as wide and taller, its contents listed (a full row on a phone).
   *
   * @default false
   */
  readonly featured?: boolean;
  /**
   * Seconds until it leaves the store: a timer chip on the tile, urgent in its last hour.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly endsIn?: number | null;
}

/**
 * A section of a store (Featured, Daily, Bundles), its offers in order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StoreSection {
  /**
   * Its key: its element id, <id>/Sections/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its heading.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Seconds until it refreshes (a daily rotation): a timer beside the heading.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly endsIn?: number | null;
  /**
   * Its offers, in the order shown.
   *
   * @default []
   */
  readonly offers?: readonly StoreOffer[];
}

/**
 * One currency in a store's or a pass's purse.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface WalletEntry {
  /**
   * Its key: its element id, so its probe path is <id>/Wallet/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name: what a screen reader says, and the style's icon of that name (gem, coin) when it has no art.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How much the player has.
   *
   * @default 0
   */
  readonly amount?: number;
  /**
   * Its art (art kind currency); null draws the style's icon of its name, else a coin or a gem drawn as shapes.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * It can be bought: a plus after it, answered as topUp.
   *
   * @default false
   */
  readonly topUp?: boolean;
}
