// The story kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { BarkView, StoryLine, Texture } from "@origingame/engine";

/**
 * A conversation's backlog, as og.story.history answers it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BacklogDesc {
  /**
   * The lines, oldest first (og.story.history).
   *
   * @default []
   */
  readonly lines?: readonly StoryLine[];
  /**
   * Its title.
   *
   * @default "Log"
   */
  readonly title?: string;
  /**
   * Close (and the back action).
   *
   * @default "Close"
   */
  readonly closeLabel?: string;
  /**
   * A voiced line's replay button, said to a screen reader.
   *
   * @default "Play voice"
   */
  readonly replayLabel?: string;
  /**
   * With no lines yet.
   *
   * @default "Nothing has been said yet."
   */
  readonly emptyLabel?: string;
}

/** What the backlog's player did this tick. */
export interface BacklogResult {
  /** Close or back was pressed: hide it. */
  readonly closed: boolean;
  /** A line's voice to play again: its StoryLine.seq (its voice is StoryLine.voice). */
  readonly replay: number | null;
}

/**
 * The barks being said, as og.story.barks answers them.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BarksDesc {
  /**
   * The barks (og.story.barks), each drawn over its entity.
   *
   * @default []
   */
  readonly barks?: readonly BarkView[];
  /**
   * World units over each speaker's point.
   *
   * @default 2.2
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * A bark of this priority or higher is a shout (bold, a jagged edge).
   *
   * @default 5
   */
  readonly shoutPriority?: number;
  /**
   * A speaker off screen keeps its bubble on the edge.
   *
   * @default false
   */
  readonly clamp?: boolean;
  /**
   * A bubble's widest; 0 is 260.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly maxWidth?: number;
}

/**
 * One event picture (CG) of the gallery, as the save holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CgEntry {
  /**
   * Its key: its tile's element id (<id>/Grid/Line#<n>/<key>).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its title under the tile and in the viewer; a locked one shows lockedLabel instead.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The chapter it belongs to: the tabs filter by it; empty shows it under every tab.
   *
   * @default ""
   */
  readonly chapter?: string;
  /**
   * Its picture (art kind cg); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its other versions (an expression, the night), the viewer's pips stepping through them after image.
   *
   * @default []
   */
  readonly variants?: readonly Texture[];
  /**
   * Not seen yet: the tile is veiled with a lock and its hint, and the viewer will not open it.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * How to unlock it, under a locked tile (Clear Mira's route); empty shows the lock alone.
   *
   * @default ""
   */
  readonly hint?: string;
  /**
   * Unlocked since the gallery was last opened: a New mark on the tile.
   *
   * @default false
   */
  readonly fresh?: boolean;
}

/**
 * A visual novel's CG gallery: the chapters as tabs, the pictures as tiles with the locked ones veiled, and the viewer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CgGalleryDesc {
  /**
   * The gallery's title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The chapters; fewer than two shows no tabs.
   *
   * @default []
   */
  readonly chapters?: readonly GalleryChapter[];
  /**
   * The chapter shown; empty all.
   *
   * @default ""
   */
  readonly chapter?: string;
  /**
   * Every picture, in the game's order.
   *
   * @default []
   */
  readonly entries?: readonly CgEntry[];
  /**
   * The picture open in the viewer; empty shows the grid.
   *
   * @default ""
   */
  readonly viewing?: string;
  /**
   * The version open in the viewer: 0 the image, then each of variants.
   *
   * @default 0
   */
  readonly variant?: number;
  /**
   * The count's words, {n} seen of {total}.
   *
   * @default "{n} / {total}"
   */
  readonly countLabel?: string;
  /**
   * A locked picture's title.
   *
   * @default "???"
   */
  readonly lockedLabel?: string;
  /**
   * The New mark.
   *
   * @default "New"
   */
  readonly newLabel?: string;
  /**
   * The viewer's close.
   *
   * @default "Close"
   */
  readonly closeLabel?: string;
  /**
   * The viewer's previous picture.
   *
   * @default "Previous"
   */
  readonly prevLabel?: string;
  /**
   * The viewer's next picture.
   *
   * @default "Next"
   */
  readonly nextLabel?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface CgGalleryResult {
  /** Another chapter's tab was pressed (or the tab actions stepped). */
  readonly chapter: string | null;
  /** An unlocked tile was pressed: open it in the viewer (at variant 0). */
  readonly open: string | null;
  /** A locked tile was pressed: it shakes. */
  readonly refused: string | null;
  /** The viewer's close, Back, or a press on the picture while its chrome is hidden. */
  readonly close: boolean;
  /**
   * The viewer's previous or next (or Left and Right): the unlocked picture to open, wrapping, skipping the locked.
   */
  readonly step: string | null;
  /** A variant's pip was pressed (or Up and Down stepped). */
  readonly variant: number | null;
}

/**
 * One chapter of a CG gallery or a route chart: its tab, or its column's heading.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GalleryChapter {
  /**
   * Its key: its tab's element id (<id>/Tabs/Row/<key>).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its word (Chapter 1, Common route).
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A visual novel's route flowchart: the scenes on their grid, the branches between them, the detail of the selected scene.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RouteChartDesc {
  /**
   * The chart's title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A heading over each column, in order; empty shows none.
   *
   * @default []
   */
  readonly chapters?: readonly GalleryChapter[];
  /**
   * The scenes.
   *
   * @default []
   */
  readonly scenes?: readonly RouteScene[];
  /**
   * The branches.
   *
   * @default []
   */
  readonly links?: readonly RouteLink[];
  /**
   * The scene selected: its detail at the side (under the chart on a phone).
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * A locked scene's title.
   *
   * @default "???"
   */
  readonly lockedLabel?: string;
  /**
   * The detail's jump to the scene.
   *
   * @default "Jump here"
   */
  readonly jumpLabel?: string;
  /**
   * The current scene's mark.
   *
   * @default "Now"
   */
  readonly currentLabel?: string;
  /**
   * The count's words, {n} seen of {total}.
   *
   * @default "{n} / {total} scenes"
   */
  readonly seenLabel?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface RouteChartResult {
  /** Another scene's card was pressed: select it. */
  readonly selected: string | null;
  /** The detail's jump was pressed on a seen scene: the game loads it. */
  readonly jump: string | null;
  /** A locked scene was pressed: it shakes. */
  readonly refused: string | null;
}

/**
 * One branch between two scenes.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RouteLink {
  /**
   * The scene it leaves.
   *
   * @default ""
   */
  readonly from?: string;
  /**
   * The scene it reaches.
   *
   * @default ""
   */
  readonly to?: string;
  /**
   * The choice that takes it, on the line at the fork; empty shows none.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Taken in a play: drawn lit and thicker; else thin and dim.
   *
   * @default false
   */
  readonly taken?: boolean;
}

/**
 * One scene of a route chart, placed by the game on the chart's grid.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RouteScene {
  /**
   * Its key: its card's element id (<id>/Chart/Map/<key>).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its title on its card; a locked one shows lockedLabel instead.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its thumbnail (art kind cg); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its column, left to right in story order (mirrored right to left).
   *
   * @default 0
   */
  readonly column?: number;
  /**
   * Its lane, top to bottom: a branch takes another lane.
   *
   * @default 0
   */
  readonly lane?: number;
  /**
   * Played: its card is lit and it can be jumped to.
   *
   * @default false
   */
  readonly seen?: boolean;
  /**
   * Not reached: a veiled card with a lock; pressing it is refused.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Where the story stands now: a ring in the focus colour and a mark.
   *
   * @default false
   */
  readonly current?: boolean;
  /**
   * An ending's word (Good end, Bad end); empty is no ending. Its card carries a band in the ending's colour.
   *
   * @default ""
   */
  readonly ending?: string;
  /**
   * With ending: a good ending's colour (success), else danger.
   *
   * @default false
   */
  readonly good?: boolean;
}
