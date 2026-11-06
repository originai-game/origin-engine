// The rhythm kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, Paint, Texture } from "@origingame/engine";

/**
 * A combo counter.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ComboDesc {
  /**
   * Hits in the combo; under 2 it hides (a break shakes it off).
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * A score multiplier shown beside it (×4); 0 or 1 hides it.
   *
   * @default 0
   */
  readonly multiplier?: number;
  /**
   * The combo's time left to the next hit, 0 to 1, as a draining bar.
   *
   * @default null
   */
  readonly decay?: number | null;
  /**
   * The word under it.
   *
   * @default "Combo"
   */
  readonly label?: string;
  /**
   * The word under it as a break shakes the count off.
   *
   * @default "Break"
   */
  readonly breakLabel?: string;
  /**
   * Counts that celebrate (a burst and a bigger pop); empty is 10, 25, 50, 100, then every 100.
   *
   * @default []
   */
  readonly milestones?: readonly number[];
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "right"
   */
  readonly anchor?: Anchor | null;
}

/**
 * Judgement pops.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface JudgementDesc {
  /**
   * The latest judgement; a later at pops again.
   *
   * @default null
   */
  readonly last?: JudgementEvent | null;
  /**
   * The early word.
   *
   * @default "Early"
   */
  readonly earlyLabel?: string;
  /**
   * The late word.
   *
   * @default "Late"
   */
  readonly lateLabel?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "center"
   */
  readonly anchor?: Anchor | null;
  /**
   * Moves it down from its anchor (above the hit line: negative).
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly offsetY?: number;
}

/**
 * A judgement of a hit.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface JudgementEvent {
  /**
   * Its word (PERFECT, GREAT, MISS).
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * 0 the best, then down: 0 gold, 1 success, 2 info, 3 warn, 4 and on danger (a miss).
   *
   * @default 0
   */
  readonly tier?: number;
  /**
   * How early (negative) or late (positive) it was; 0 shows nothing.
   *
   * @default 0
   * @unit ms (milliseconds)
   */
  readonly offsetMs?: number;
  /**
   * og.ui.tick() when it happened.
   *
   * @default 0
   */
  readonly at?: number;
}

/**
 * A lane's key on og.ui.laneKeys.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LaneKey {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The key's letter (D, F, J, K); empty uses action's glyph.
   *
   * @default ""
   */
  readonly hotkey?: string;
  /**
   * An input action whose glyph on the device used last it shows when hotkey is empty.
   *
   * @default ""
   */
  readonly action?: string;
  /**
   * The lane's centre on the screen, in CSS px from the viewport's left.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x?: number;
  /**
   * The lane's colour; null is the accent.
   *
   * @default null
   */
  readonly color?: Paint | null;
}

/**
 * A rhythm game's lane keys.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LaneKeysDesc {
  /**
   * The lanes, left to right.
   *
   * @default []
   */
  readonly lanes?: readonly LaneKey[];
  /**
   * The hit line, in CSS px from the viewport's top: the caps hang under it.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y?: number;
  /**
   * A cap's width; 0 is 56.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width?: number;
  /**
   * The lanes held now (their keys).
   *
   * @default []
   */
  readonly pressed?: readonly string[];
  /**
   * The lanes whose press hit a note this tick: a flash in the lane's colour.
   *
   * @default []
   */
  readonly hit?: readonly string[];
}

/**
 * One chart of a song.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SongDifficulty {
  /**
   * Its key (easy, expert).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name on its chip (Easy, Expert).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its level, the chip's numeral.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * Not playable yet: the chip dimmed with a lock.
   *
   * @default false
   */
  readonly locked?: boolean;
}

/**
 * One song on the wheel.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SongEntry {
  /**
   * Its key: its row's element id, so its probe path is <id>/Wheel/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its title, the row's reading and the detail's heading.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its artist, under the title.
   *
   * @default ""
   */
  readonly artist?: string;
  /**
   * Its cover art (art kind jacket), square; null draws the style's placeholder.
   *
   * @default null
   */
  readonly jacket?: Texture | null;
  /**
   * Its tempo; 0 shows none.
   *
   * @default 0
   */
  readonly bpm?: number;
  /**
   * Its length; 0 shows none.
   *
   * @default 0
   * @unit s (seconds of simulation time)
   */
  readonly seconds?: number;
  /**
   * Its charts, easiest first.
   *
   * @default []
   */
  readonly difficulties?: readonly SongDifficulty[];
  /**
   * The best score on the selected chart; 0 shows none.
   *
   * @default 0
   */
  readonly best?: number;
  /**
   * The best grade on the selected chart (S, A); empty shows none.
   *
   * @default ""
   */
  readonly grade?: string;
  /**
   * The selected chart was cleared with a full combo: a badge beside the grade.
   *
   * @default false
   */
  readonly fullCombo?: boolean;
  /**
   * The best accuracy on the selected chart, 0 to 1; a negative value shows none.
   *
   * @default -1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly accuracy?: number;
  /**
   * Not unlocked: its row dimmed with a lock, and Play refused.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Not played yet: a New tag on its row.
   *
   * @default false
   */
  readonly isNew?: boolean;
}

/**
 * The selected song's preview, which the game plays (og.audio).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SongPreview {
  /**
   * The preview is playing: the bar's play mark is a pause and its played part lit.
   *
   * @default false
   */
  readonly playing?: boolean;
  /**
   * How far into the song the preview is.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly position?: number;
  /**
   * The song's waveform, its peaks 0 to 1 across its length (64 to 128 of them); empty draws a flat line.
   *
   * @default []
   */
  readonly peaks?: readonly number[];
}

/**
 * A song select: the songs, the selected one and chart, the preview.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SongSelectDesc {
  /**
   * The songs, in the wheel's order.
   *
   * @default []
   */
  readonly songs?: readonly SongEntry[];
  /**
   * The song the wheel stands on (the game's: it passes SongSelectResult.selected back).
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * The selected chart's key.
   *
   * @default ""
   */
  readonly difficulty?: string;
  /**
   * The selected song's preview.
   *
   * @default {}
   */
  readonly preview?: SongPreview;
  /**
   * The tempo's label.
   *
   * @default "BPM"
   */
  readonly bpmLabel?: string;
  /**
   * The length's label.
   *
   * @default "Length"
   */
  readonly lengthLabel?: string;
  /**
   * The best score's label.
   *
   * @default "Best score"
   */
  readonly bestLabel?: string;
  /**
   * The grade's label.
   *
   * @default "Grade"
   */
  readonly gradeLabel?: string;
  /**
   * The full combo badge.
   *
   * @default "Full combo"
   */
  readonly fullComboLabel?: string;
  /**
   * The accuracy ring's caption.
   *
   * @default "Accuracy"
   */
  readonly accuracyLabel?: string;
  /**
   * The call to action.
   *
   * @default "Play"
   */
  readonly playLabel?: string;
  /**
   * The preview's play toggle, to a screen reader.
   *
   * @default "Preview"
   */
  readonly previewLabel?: string;
  /**
   * A new song's tag.
   *
   * @default "New"
   */
  readonly newLabel?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface SongSelectResult {
  /** Another song was focused or pressed: stand the wheel on it (and start its preview). */
  readonly selected: string | null;
  /** Another chart was pressed, or the tab actions (Q and E, the bumpers) stepped to it. */
  readonly difficulty: string | null;
  /**
   * Play was pressed, or the selected song's row pressed again, or confirm on it: start the chart.
   */
  readonly play: boolean;
  /** The preview's toggle was pressed: play or pause it. */
  readonly preview: boolean;
  /** A locked song or chart was asked to play: it shook. */
  readonly refused: string | null;
}
