// The puzzle kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Texture } from "@origingame/engine";

/**
 * A level's goals.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GoalChipsDesc {
  /**
   * The goals, in the order the level states them.
   *
   * @default []
   */
  readonly goals?: readonly LevelGoal[];
  /**
   * Counts only, as a row of icons with a number each (a phone's top strip): the label goes to a screen reader.
   *
   * @default false
   */
  readonly compact?: boolean;
  /**
   * What a screen reader calls them.
   *
   * @default "Goals"
   */
  readonly label?: string;
  /**
   * A goal's count to a screen reader ({have} of {need}).
   *
   * @default "{have} of {need}"
   */
  readonly progressLabel?: string;
  /**
   * What a goal met adds.
   *
   * @default "Done"
   */
  readonly doneLabel?: string;
}

/**
 * One goal of a level (collect 20 blue gems, break 6 ice tiles).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LevelGoal {
  /**
   * Its key: its element id, <id>/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What to do (Collect Blue Gems).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * How many are done.
   *
   * @default 0
   */
  readonly have?: number;
  /**
   * How many it takes.
   *
   * @default 0
   */
  readonly need?: number;
  /**
   * Its piece's art (art kind item); null draws icon or a gem.
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
}

/**
 * A saga map: its levels in order, where the player stands, and its art.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SagaMapDesc {
  /**
   * The levels, first to last.
   *
   * @default []
   */
  readonly nodes?: readonly SagaNode[];
  /**
   * The level the player stands on: the avatar's pin over it, lit, the path lit up to it; the map opens centred on it.
   *
   * @default ""
   */
  readonly current?: string;
  /**
   * The player's avatar in the pin (art kind avatar); null draws a figure.
   *
   * @default null
   */
  readonly avatar?: Texture | null;
  /**
   * The map's painted art behind the path, across the whole scroll (art kind map); null draws a dusk gradient.
   *
   * @default null
   */
  readonly backdrop?: Texture | null;
  /**
   * Space between levels along the path; 0 is 150 (scaled with the screen).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly spacing?: number;
  /**
   * What a screen reader calls the map.
   *
   * @default "Levels"
   */
  readonly label?: string;
  /**
   * A level's name to a screen reader ({label} is its badge).
   *
   * @default "Level {label}"
   */
  readonly levelLabel?: string;
  /**
   * A boss level's name to a screen reader ({label} is its badge).
   *
   * @default "Boss, level {label}"
   */
  readonly bossLabel?: string;
  /**
   * Its stars to a screen reader ({n} earned of {max}).
   *
   * @default "{n} of {max} stars"
   */
  readonly starsLabel?: string;
  /**
   * What a locked level adds.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
  /**
   * What the current level adds.
   *
   * @default "You are here"
   */
  readonly currentLabel?: string;
  /**
   * Room at the map's foot that the game's own chrome takes (a Play button docked at the thumb): the view of the path stops above it, so no level sits under the button, while the map's art runs on behind it; 0 is none.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly footInset?: number;
}

/** What the map did this tick. */
export interface SagaMapResult {
  /** A level pressed that is open: start it (or show its goals first). */
  readonly picked: string | null;
  /** A locked level pressed: it shook. */
  readonly refused: string | null;
  /** The level under the pointer or the focus. */
  readonly hovered: string | null;
}

/**
 * One level on a saga map, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SagaNode {
  /**
   * Its key, stable for the run: its element id, so its probe path is <id>/View/Path/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What its badge reads (47); empty is its place counted from 1.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Stars earned on it, 0 to maxStars.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Stars it can give; 0 shows none (a boss gives none).
   *
   * @default 3
   */
  readonly maxStars?: number;
  /**
   * Not reached yet: dimmed with a lock, and a press shakes and is refused.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * A boss level: a larger badge in the danger ink, its portrait inside.
   *
   * @default false
   */
  readonly boss?: boolean;
  /**
   * A boss's portrait (art kind portrait); null draws a crown.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Where it sits across the path, -1 to 1 (the map's art decides); 0 lets the map wind it.
   *
   * @default 0
   * @range -1..1
   */
  readonly lane?: number;
}
