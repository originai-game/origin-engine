// The photo kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Texture } from "@origingame/engine";

/**
 * One slider of photo mode (field of view, roll, focus distance, aperture, exposure, vignette).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PhotoControl {
  /**
   * Its key: its row's element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The tab it stands under.
   *
   * @default ""
   */
  readonly tab?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value, from game state.
   *
   * @default 0
   */
  readonly value?: number;
  /**
   * Its low end.
   *
   * @default 0
   */
  readonly min?: number;
  /**
   * Its high end.
   *
   * @default 1
   */
  readonly max?: number;
  /**
   * Its step; 0 is continuous.
   *
   * @default 0
   */
  readonly step?: number;
  /**
   * A unit after its value (°, m, EV; a word's unit stands after a space); `%` writes a 0-to-1 value as a percent.
   *
   * @default ""
   */
  readonly unit?: string;
  /**
   * Decimals in its value.
   *
   * @default 0
   */
  readonly digits?: number;
  /**
   * A centre the fill grows from (roll 0, exposure 0); null fills from the low end.
   *
   * @default null
   */
  readonly origin?: number | null;
}

/**
 * A filter or a frame: its tile shows the picture as it would come out.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PhotoLook {
  /**
   * Its key: its tile's element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name under the tile.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its thumbnail (the game's own render of the shot through it); null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * A wash over the thumbnail (`#rrggbbaa`) where the game has no render of its own; empty is none.
   *
   * @default ""
   */
  readonly tint?: string;
}

/**
 * Photo mode's controls over the game's paused, free camera: the tabs, the sliders, the filters and frames, the grid and the shutter.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PhotoModeDesc {
  /**
   * The tabs, start to end.
   *
   * @default []
   */
  readonly tabs?: readonly PhotoTab[];
  /**
   * The tab shown.
   *
   * @default ""
   */
  readonly tab?: string;
  /**
   * Every slider, each under its tab.
   *
   * @default []
   */
  readonly controls?: readonly PhotoControl[];
  /**
   * The filters, for the tab `filters`; the first is usually None.
   *
   * @default []
   */
  readonly filters?: readonly PhotoLook[];
  /**
   * The filter applied (the game draws it).
   *
   * @default ""
   */
  readonly filter?: string;
  /**
   * The filter's strength, 0 to 1, a slider under the tiles; null shows none.
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly strength?: number | null;
  /**
   * The frames, for the tab `frames`.
   *
   * @default []
   */
  readonly frames?: readonly PhotoLook[];
  /**
   * The frame chosen.
   *
   * @default ""
   */
  readonly frame?: string;
  /**
   * The chosen frame's art drawn over the whole picture (transparent where the picture shows); null draws none.
   *
   * @default null
   */
  readonly frameImage?: Texture | null;
  /**
   * The rule of thirds over the picture.
   *
   * @default false
   */
  readonly grid?: boolean;
  /**
   * The controls are hidden for a clean shot: only a quiet hint stays, and any press, Back or the hide action brings them back.
   *
   * @default false
   */
  readonly hidden?: boolean;
  /**
   * How long the shutter's flash lasts after Capture.
   *
   * @default 260
   * @unit ms (milliseconds)
   */
  readonly flashMs?: number;
  /**
   * The panel's title.
   *
   * @default "Photo mode"
   */
  readonly title?: string;
  /**
   * The shutter's name.
   *
   * @default "Capture"
   */
  readonly captureLabel?: string;
  /**
   * Hide UI.
   *
   * @default "Hide UI"
   */
  readonly hideLabel?: string;
  /**
   * The grid's toggle.
   *
   * @default "Grid"
   */
  readonly gridLabel?: string;
  /**
   * Reset the sliders.
   *
   * @default "Reset"
   */
  readonly resetLabel?: string;
  /**
   * Leave photo mode.
   *
   * @default "Exit"
   */
  readonly exitLabel?: string;
  /**
   * The filter strength's name.
   *
   * @default "Strength"
   */
  readonly strengthLabel?: string;
  /**
   * The hint while hidden.
   *
   * @default "Press any key to show the controls"
   */
  readonly hiddenHint?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface PhotoModeResult {
  /** Another tab was pressed (or the tab actions stepped). */
  readonly tab: string | null;
  /** A slider moved. */
  readonly set: PhotoSet | null;
  /** A filter's tile was pressed. */
  readonly filter: string | null;
  /** The filter strength moved. */
  readonly strength: number | null;
  /** A frame's tile was pressed. */
  readonly frame: string | null;
  /** The grid's toggle was pressed: flip it. */
  readonly grid: boolean;
  /**
   * Hide UI was pressed (or the details action), or the hidden controls were asked back: flip `hidden`.
   */
  readonly hide: boolean;
  /**
   * The shutter was pressed, or confirm while hidden: the game takes the shot (the flash plays here).
   */
  readonly capture: boolean;
  /** Reset was pressed: the sliders to their defaults. */
  readonly reset: boolean;
  /** Exit or Back was pressed (Back while hidden only brings the controls back). */
  readonly exit: boolean;
}

/** A slider moved. */
export interface PhotoSet {
  /** The control. */
  readonly key: string;
  /** Its new value. */
  readonly value: number;
}

/**
 * One tab of photo mode's controls (Camera, Lens, Filters, Frames).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PhotoTab {
  /**
   * Its key: its tab's element id; the controls' `tab`. The keys `filters` and `frames` show the filter and frame tiles.
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
