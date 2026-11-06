// The flight kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, Paint } from "@origingame/engine";

/**
 * A flight HUD.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FlightHudDesc {
  /**
   * Pitch, nose up positive.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly pitch?: number;
  /**
   * Roll, right wing down positive.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly roll?: number;
  /**
   * Heading, clockwise from north.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly heading?: number;
  /**
   * Airspeed.
   *
   * @default 0
   */
  readonly speed?: number;
  /**
   * Altitude.
   *
   * @default 0
   */
  readonly altitude?: number;
  /**
   * The speed's unit.
   *
   * @default "kt"
   */
  readonly speedUnit?: string;
  /**
   * The altitude's unit.
   *
   * @default "ft"
   */
  readonly altitudeUnit?: string;
  /**
   * Climb rate (shown by the altitude).
   *
   * @default 0
   */
  readonly verticalSpeed?: number;
  /**
   * The flight path marker's offset from the boresight, across.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly pathX?: number;
  /**
   * Its offset, down.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly pathY?: number;
  /**
   * The HUD's ink; null is the style's primary, lifted toward white.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * The vertical field of view the ladder spans (degrees of pitch per screen); 0 is 40°.
   *
   * @default 0
   * @range 0..
   * @unit rad (radians)
   */
  readonly fov?: number;
  /**
   * Where it sits; the default is the centre.
   *
   * @default "center"
   */
  readonly anchor?: Anchor;
}
