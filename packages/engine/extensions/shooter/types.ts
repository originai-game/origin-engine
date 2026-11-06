// The shooter kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, Paint, Rarity, Texture } from "@origingame/engine";

/**
 * A weapon's ammunition.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AmmoDesc {
  /**
   * Rounds in the magazine.
   *
   * @default 0
   */
  readonly magazine?: number;
  /**
   * Rounds a magazine holds.
   *
   * @default 0
   */
  readonly magazineMax?: number;
  /**
   * Rounds in reserve; null is infinite (∞).
   *
   * @default null
   */
  readonly reserve?: number | null;
  /**
   * The weapon's name; empty is none.
   *
   * @default ""
   */
  readonly weapon?: string;
  /**
   * The weapon's icon.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its fire mode (AUTO, BURST); empty is none.
   *
   * @default ""
   */
  readonly fireMode?: string;
  /**
   * A reload's progress 0 to 1; null is not reloading.
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly reloading?: number | null;
  /**
   * The reload word.
   *
   * @default "Reloading"
   */
  readonly reloadLabel?: string;
  /**
   * At or under this fraction of a magazine it is low: warn, and the reload hint shows.
   *
   * @default 0.25
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly low?: number;
  /**
   * Under low: the hint (R Reload); empty is none.
   *
   * @default ""
   */
  readonly reloadHint?: string;
  /**
   * The magazine as a row of rounds when it holds 40 or fewer (the default).
   *
   * @default true
   */
  readonly pips?: boolean;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottomRight"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A crosshair and its hit markers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CrosshairDesc {
  /**
   * Its shape.
   *
   * @default "cross"
   */
  readonly style?: CrosshairStyle;
  /**
   * How far the lines stand off for the weapon's spread (moves on a fast spring).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly spread?: number;
  /**
   * A line's length; 0 is 8.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly length?: number;
  /**
   * A line's width; 0 is 2.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly thickness?: number;
  /**
   * The gap at rest; 0 is 6.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Its ink; null is white.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * A dark outline round each line (the default).
   *
   * @default true
   */
  readonly outline?: boolean;
  /**
   * A centre dot with the lines.
   *
   * @default false
   */
  readonly dot?: boolean;
  /**
   * og.ui.tick() of the last hit: a marker flashes for it.
   *
   * @default null
   */
  readonly hitAt?: number | null;
  /**
   * What the last hit was.
   *
   * @default "hit"
   */
  readonly hit?: HitKind;
  /**
   * The aim is on an enemy: the ink turns danger.
   *
   * @default false
   */
  readonly onTarget?: boolean;
  /**
   * Where it sits; the default is the centre. null builds it in place
   * (inside a split-screen half's own group, centred by its parent).
   *
   * @default "center"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A crosshair's shape.
 * - `"cross"`: Four lines round a gap (the default).
 * - `"dot"`: A dot.
 * - `"circle"`: A ring with a dot.
 * - `"chevron"`: A chevron under the aim point.
 * - `"tShape"`: Three lines, none above (Valorant's T).
 */
export type CrosshairStyle = "cross" | "dot" | "circle" | "chevron" | "tShape";

/**
 * Damage direction indicators.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DamageDirectionDesc {
  /**
   * Recent hits.
   *
   * @default []
   */
  readonly hits?: readonly DamageHit[];
  /**
   * How long an indicator shows; 0 is 1400.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * Its distance from the centre; 0 is a fifth of the surface's shorter side.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
  /**
   * Its ink; null is danger.
   *
   * @default null
   */
  readonly color?: Paint | null;
}

/**
 * A hit from a direction.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DamageHit {
  /**
   * Where it came from, clockwise from straight ahead.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly angle?: number;
  /**
   * og.ui.tick() when it landed.
   *
   * @default 0
   */
  readonly at?: number;
  /**
   * How hard, 0 to 1 (its width and opacity).
   *
   * @default 1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly strength?: number;
}

/**
 * What a hit marker shows.
 * - `"hit"`: A hit: four white ticks.
 * - `"headshot"`: A critical or headshot: larger, in gold.
 * - `"kill"`: A kill: larger, in danger, held longer.
 */
export type HitKind = "hit" | "headshot" | "kill";

/**
 * A killfeed.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface KillfeedDesc {
  /**
   * The latest entries, oldest first.
   *
   * @default []
   */
  readonly entries?: readonly KillfeedEntry[];
  /**
   * The most lines; 0 is 5 (3 on a phone).
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * How long a line stays; 0 is 6000.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * Opens its own anchored group there (top right; on a phone in portrait top centre, stacked under the compass and the timer built before it, in landscape mid-right); null builds it in place.
   *
   * @default "topRight"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A line of a killfeed.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface KillfeedEntry {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Who got the kill; empty for a fall or the zone.
   *
   * @default ""
   */
  readonly killer?: string;
  /**
   * The killer's team (its colour).
   *
   * @default 0
   */
  readonly killerTeam?: number;
  /**
   * Who fell.
   *
   * @default ""
   */
  readonly victim?: string;
  /**
   * The victim's team.
   *
   * @default 0
   */
  readonly victimTeam?: number;
  /**
   * The weapon's icon.
   *
   * @default ""
   */
  readonly weapon?: string;
  /**
   * The weapon in words, when it has no icon.
   *
   * @default ""
   */
  readonly weaponText?: string;
  /**
   * A headshot mark.
   *
   * @default false
   */
  readonly headshot?: boolean;
  /**
   * An assister, after the killer (+ name).
   *
   * @default ""
   */
  readonly assist?: string;
  /**
   * The player is in it: the line is highlighted.
   *
   * @default false
   */
  readonly local?: boolean;
  /**
   * og.ui.tick() when it happened.
   *
   * @default 0
   */
  readonly at?: number;
}

/**
 * One attachment point of the selected item, and what is on it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadoutAttachment {
  /**
   * Its key: its callout's and its row's element id (<id>/Stage/Callouts/<key>, <id>/Side/Attachments/List/<key>).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The point's name, the callout's eyebrow (Barrel, Optic).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * What is on it (Vanguard Suppressor); empty is nothing, the point's emptyLabel shown, and it has no row.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its art (art kind itemIcon); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: the row's band and the callout's rim.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * The tier's word on the row's band (Rare); empty shows none.
   *
   * @default ""
   */
  readonly rarityLabel?: string;
  /**
   * Where the point is on the item's art, across it, 0 to 1: the leader line's end.
   *
   * @default 0.5
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly x?: number;
  /**
   * Where the point is, down it, 0 to 1; above the middle its callout stands above the art, else below.
   *
   * @default 0.5
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly y?: number;
}

/**
 * A loadout screen: the loadouts' tabs, the slots, the selected item large with its attachment points, its figures and its attachments.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadoutDesc {
  /**
   * The screen's title (Loadout).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The loadouts' names, one tab each (Loadout 1); fewer than two shows no tabs.
   *
   * @default []
   */
  readonly loadouts?: readonly string[];
  /**
   * The loadout shown, an index of loadouts.
   *
   * @default 0
   */
  readonly loadout?: number;
  /**
   * The purse at the end of the head; null shows none.
   *
   * @default null
   */
  readonly balance?: number | null;
  /**
   * The purse's coin; null draws the style's coin.
   *
   * @default null
   */
  readonly currencyImage?: Texture | null;
  /**
   * The slots, top to bottom.
   *
   * @default []
   */
  readonly slots?: readonly LoadoutSlot[];
  /**
   * The selected slot: its item stands large in the middle.
   *
   * @default ""
   */
  readonly slot?: string;
  /**
   * The selected item's art, large (art kind item, a side view on transparent ground); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * The item's art's width over its height, so the attachment points land on it as drawn (fit inside its box); 0 takes the box's own.
   *
   * @default 0
   */
  readonly aspect?: number;
  /**
   * The selected item's name, in the display face.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The words under its name, divided by rules (Assault rifle, Rare, Lvl 12).
   *
   * @default []
   */
  readonly subtitle?: readonly string[];
  /**
   * Its tier: the subtitle's tier word takes its colour when rarityWord names the word's index.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Which of subtitle is the tier's word, -1 none.
   *
   * @default -1
   */
  readonly rarityWord?: number;
  /**
   * A line under the subtitle; empty shows none.
   *
   * @default ""
   */
  readonly description?: string;
  /**
   * The item's attachment points: callouts on leader lines around its art on a desktop, rows at the side.
   *
   * @default []
   */
  readonly attachments?: readonly LoadoutAttachment[];
  /**
   * The attachment point picked (its callout and row lit); empty none.
   *
   * @default ""
   */
  readonly attachment?: string;
  /**
   * The attachment points the item has in all, for the count (5/5); 0 counts the given points.
   *
   * @default 0
   */
  readonly capacity?: number;
  /**
   * The item's figures.
   *
   * @default []
   */
  readonly stats?: readonly LoadoutStat[];
  /**
   * The figures' heading.
   *
   * @default "Weapon stats"
   */
  readonly statsLabel?: string;
  /**
   * The attachments' heading.
   *
   * @default "Attachments"
   */
  readonly attachmentsLabel?: string;
  /**
   * An empty slot's or point's word.
   *
   * @default "Empty"
   */
  readonly emptyLabel?: string;
  /**
   * The remove button's name, before the attachment's.
   *
   * @default "Remove"
   */
  readonly removeLabel?: string;
  /**
   * A locked slot's word.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface LoadoutResult {
  /** Another loadout's tab was pressed (or the tab actions stepped). */
  readonly loadout: number | null;
  /** Another slot was pressed or reached by keys. */
  readonly slot: string | null;
  /** An attachment point was pressed (its callout or its row): the game opens its choices. */
  readonly attachment: string | null;
  /** An attachment's remove button was pressed. */
  readonly remove: string | null;
  /** A locked slot was pressed: it shakes. */
  readonly refused: string | null;
}

/**
 * One slot of a loadout (Primary, Secondary, Lethal, Tactical, Perk 1), as the game holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadoutSlot {
  /**
   * Its key: its rail entry's element id, so its probe path is <id>/Slots/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What the slot is, the entry's eyebrow (Primary).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * What is in it (AR-7 Raven); empty is an empty slot, its emptyLabel shown.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A line under the name (Lvl 12); empty shows none.
   *
   * @default ""
   */
  readonly detail?: string;
  /**
   * Its art (art kind itemIcon, or a weapon's side view); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: the art's rim.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * Not available yet: dimmed with a lock; pressing it is refused.
   *
   * @default false
   */
  readonly locked?: boolean;
}

/**
 * One of the selected item's figures, with what its attachments change.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadoutStat {
  /**
   * Its key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name (Damage).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value with the attachments.
   *
   * @default 0
   */
  readonly value?: number;
  /**
   * What the attachments add (negative: take away); 0 shows none.
   *
   * @default 0
   */
  readonly delta?: number;
  /**
   * The bar's whole; 0 is 100.
   *
   * @default 0
   * @range 0..
   */
  readonly max?: number;
}

/**
 * A low-health and hit vignette.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LowHealthDesc {
  /**
   * Health as a fraction, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly health?: number;
  /**
   * Under this the vignette shows, deepening to 0.
   *
   * @default 0.35
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly threshold?: number;
  /**
   * og.ui.tick() of the last hit: a quick red flash at the edges.
   *
   * @default null
   */
  readonly hitAt?: number | null;
  /**
   * Its ink; null is a deep red.
   *
   * @default null
   */
  readonly color?: Paint | null;
}

/**
 * A scoreboard column.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScoreColumn {
  /**
   * Its key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its heading (K, D, A, Score).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its width; 0 is 56.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width?: number;
}

/**
 * A player's row on a scoreboard.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScoreRow {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The player's name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The values, one per column.
   *
   * @default []
   */
  readonly values?: readonly string[];
  /**
   * The player: highlighted.
   *
   * @default false
   */
  readonly local?: boolean;
  /**
   * Dead: dimmed with a skull.
   *
   * @default false
   */
  readonly dead?: boolean;
  /**
   * Latency in ms: bars, warn past 100, danger past 200; 0 hides it.
   *
   * @default 0
   */
  readonly ping?: number;
  /**
   * A portrait (art kind portrait).
   *
   * @default null
   */
  readonly avatar?: Texture | null;
  /**
   * A rank, class or agent icon.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * The best of the match: a star.
   *
   * @default false
   */
  readonly mvp?: boolean;
}

/**
 * A team on a scoreboard.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScoreTeam {
  /**
   * Its team colour.
   *
   * @default 0
   */
  readonly team?: number;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its score.
   *
   * @default ""
   */
  readonly score?: string;
  /**
   * Its players.
   *
   * @default []
   */
  readonly rows?: readonly ScoreRow[];
}

/**
 * A scoreboard.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScoreboardDesc {
  /**
   * The title (the mode, the map).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it (the round, the time).
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * The columns after the name.
   *
   * @default []
   */
  readonly columns?: readonly ScoreColumn[];
  /**
   * The teams, each with its rows.
   *
   * @default []
   */
  readonly teams?: readonly ScoreTeam[];
  /**
   * Its width; 0 is 760 (the safe area less its gutters on a phone).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "center"
   */
  readonly anchor?: Anchor | null;
}
