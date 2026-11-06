// The sports kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, Paint, Texture } from "@origingame/engine";

/**
 * The scorers by side.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GoalTimelineDesc {
  /**
   * The events, in match order.
   *
   * @default []
   */
  readonly events?: readonly MatchEvent[];
  /**
   * Opens its own anchored group there; null builds it in place (the default).
   *
   * @default null
   */
  readonly anchor?: Anchor | null;
}

/**
 * One event on og.ui.goalTimeline.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MatchEvent {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * 0 home, 1 away.
   *
   * @default 0
   */
  readonly team?: number;
  /**
   * Who (J. Anderson).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * When, as shown (23', 90+2').
   *
   * @default ""
   */
  readonly minute?: string;
  /**
   * Its mark (an icon name: a ball for a goal, a card); empty is the ball.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * A note after the name (pen., o.g.); empty is none.
   *
   * @default ""
   */
  readonly note?: string;
}

/**
 * One row of og.ui.matchStats: a stat for both sides.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MatchStat {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, centred over the bar (Possession, Shots).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The home side's value.
   *
   * @default 0
   */
  readonly home?: number;
  /**
   * The away side's value.
   *
   * @default 0
   */
  readonly away?: number;
  /**
   * The home value as shown (58%); empty prints the value.
   *
   * @default ""
   */
  readonly homeText?: string;
  /**
   * The away value as shown; empty prints the value.
   *
   * @default ""
   */
  readonly awayText?: string;
  /**
   * More is worse (fouls, cards): the bar stays both teams' colours and neither numeral is tinted (the larger side is not lit as a lead).
   *
   * @default false
   */
  readonly worse?: boolean;
}

/**
 * Two sides' stats face to face.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MatchStatsDesc {
  /**
   * The home side (name or abbr, crest, colour).
   *
   * @default {}
   */
  readonly home?: ScorebugTeam;
  /**
   * The away side.
   *
   * @default {}
   */
  readonly away?: ScorebugTeam;
  /**
   * The rows, in order.
   *
   * @default []
   */
  readonly stats?: readonly MatchStat[];
  /**
   * A caption over the rows (Match stats, Possession); empty is none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The sides' names over their columns (a full-time board); a HUD plate leaves them out and shows the crests beside its one row.
   *
   * @default false
   */
  readonly header?: boolean;
  /**
   * Opens its own anchored group there on a HUD plate; null builds it in place (the default).
   *
   * @default null
   */
  readonly anchor?: Anchor | null;
}

/**
 * The controlled player.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PlayerPlateDesc {
  /**
   * The shirt number.
   *
   * @default ""
   */
  readonly number?: string;
  /**
   * The name (Martin).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The side's crest (art kind emblem, 1:1).
   *
   * @default null
   */
  readonly logo?: Texture | null;
  /**
   * The side's colour; null is the home team colour.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Stamina 0 to 1.
   *
   * @default 1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly stamina?: number;
  /**
   * The stamina bar's segments.
   *
   * @default 6
   */
  readonly segments?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottomLeft"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A broadcast scorebug.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScorebugDesc {
  /**
   * The home side.
   *
   * @default {}
   */
  readonly home?: ScorebugTeam;
  /**
   * The away side.
   *
   * @default {}
   */
  readonly away?: ScorebugTeam;
  /**
   * The game clock (12:34).
   *
   * @default ""
   */
  readonly clock?: string;
  /**
   * The period (Q2, 2nd, ET).
   *
   * @default ""
   */
  readonly period?: string;
  /**
   * A shot or play clock; empty is none.
   *
   * @default ""
   */
  readonly shotClock?: string;
  /**
   * A line under it (2nd & 7, Power play); empty is none.
   *
   * @default ""
   */
  readonly situation?: string;
  /**
   * Added or stoppage time beside the clock (+3), in the accent; empty is none.
   *
   * @default ""
   */
  readonly added?: string;
  /**
   * The match is over: this label (FT, Final) in a chip in place of the clock, the period and the added time, which it never shows; empty is a live clock.
   *
   * @default ""
   */
  readonly final?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "topLeft"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A team on a scorebug.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScorebugTeam {
  /**
   * Its short name (LAL).
   *
   * @default ""
   */
  readonly abbr?: string;
  /**
   * Its score.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * Its colour; null is a team colour by side.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Its crest (art kind emblem, 1:1).
   *
   * @default null
   */
  readonly logo?: Texture | null;
  /**
   * Time-outs left (pips).
   *
   * @default 0
   */
  readonly timeouts?: number;
  /**
   * Time-outs it had; 0 draws no pips.
   *
   * @default 0
   */
  readonly timeoutsMax?: number;
  /**
   * It has the ball: a mark by its score.
   *
   * @default false
   */
  readonly possession?: boolean;
  /**
   * Its full name (Northgate) where there is room for it (og.ui.matchStats' header); empty is abbr.
   *
   * @default ""
   */
  readonly name?: string;
}

/**
 * A player on og.ui.substitution.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SubPlayer {
  /**
   * The shirt number.
   *
   * @default ""
   */
  readonly number?: string;
  /**
   * The name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The portrait (art kind portrait, 1:1).
   *
   * @default null
   */
  readonly image?: Texture | null;
}

/**
 * A substitution.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SubstitutionDesc {
  /**
   * The player going off.
   *
   * @default {}
   */
  readonly off?: SubPlayer;
  /**
   * The player coming on.
   *
   * @default {}
   */
  readonly on?: SubPlayer;
  /**
   * The tab over it.
   *
   * @default "Substitution"
   */
  readonly caption?: string;
  /**
   * The word under the player going off.
   *
   * @default "Out"
   */
  readonly offLabel?: string;
  /**
   * The word under the player coming on.
   *
   * @default "In"
   */
  readonly onLabel?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottom"
   */
  readonly anchor?: Anchor | null;
}
