// The public shapes of @origingame/engine/ui: the nodes, children, events
// and props of the host elements (crates/og-ui-next/CONVENTIONS.md §Ops).
// og api gen emits types/ui.d.ts from src/ui/index.ts, which re-exports these.

import type { Entity, Texture, World } from "@origingame/engine";

declare global {
  namespace JSX {
    type Element = UiChild;
    // Tags are PascalCase imports: no intrinsic elements.
    interface IntrinsicElements {}
    interface ElementChildrenAttribute { children: {} }
  }
}

/** A node the renderer made: a handle on the UI tree's node `id`. */
export interface UiNode { readonly id: number }
/** What a child expression may give: nodes, text, nothing, lists and functions of them. */
export type UiChild = UiNode | string | number | boolean | null | undefined | UiChild[] | (() => UiChild);
/** What an event handler is given. */
export interface UiEvent {
  /** The world the UI belongs to, for writing intents and commands. */
  readonly world: World;
  readonly type: string;
  /** The pointer on the UI's surface (`onWheel`: the wheel's deltas). */
  readonly x: number;
  readonly y: number;
  /** A key's name (`onKey`) or the input action (`onAction`). */
  readonly key: string;
  /**
   * A control's new value (`onChange`); a hold's progress, 0 to 1, "0" when
   * let go early (`onHold`); the box of the node that hears a pointer event
   * on the surface, `"x y width height"` (`onPointerDown`/`Move`/`Up`,
   * `onLongPress`), so a slider maps `(x - box.x) / box.width`; `"2"` on a
   * double press's second `onPress`.
   */
  readonly value: string;
}

type Handler = ((e: UiEvent) => void) | undefined;

/** What every host element takes. Every optional prop also takes `undefined`. */
export interface HostProps {
  class?: string | undefined;
  classList?: Record<string, boolean | undefined> | undefined;
  style?: string | Record<string, string | number | undefined> | undefined;
  ref?: ((node: UiNode) => void) | undefined;
  children?: UiChild;
  onPress?: Handler;
  onTap?: Handler;
  onPointerDown?: Handler;
  onPointerMove?: Handler;
  onPointerUp?: Handler;
  onWheel?: Handler;
  onFocus?: Handler;
  onBlur?: Handler;
  onKey?: Handler;
  onChange?: Handler;
  /** A finger held still on it for 0.5 s; its release then presses nothing. `value` is its box. */
  onLongPress?: Handler;
  /** A held press, with `hold` set: its progress each frame in `value`, 0 to 1 when it completes; "0" when it lets go early. */
  onHold?: Handler;
  /** How long a press must be held for `onHold` to complete, in seconds. */
  hold?: number | undefined;
  /** Takes focus when it appears (a dialog's first control). */
  autofocus?: boolean | undefined;
  /** Its focusable descendants are one group for the arrows and the d-pad: they move among its members first; at its edge `"stop"` leaves it for the control beyond, `"wrap"` goes round inside it. */
  focusGroup?: "stop" | "wrap" | undefined;
  /** With `focusGroup`: focus coming back into it lands on the member it last had. */
  focusMemory?: boolean | undefined;
  /** Makes it draggable, carrying this data: a press moved past the slop picks it up, as do Enter, Space or `confirm` on it when focused. */
  draggable?: string | undefined;
  /** Makes it a drop target with this id: a carried node let go over it, or put down while it has the focus, drops here. While a node is carried by the keys, targets are focus stops. */
  dropTarget?: string | undefined;
  /** It was picked up: `key` its `draggable` data, `value` its box. */
  onDragStart?: Handler;
  /** It moved while carried: `key` its data, `value` the `dropTarget` under it (or focused), "" over none. */
  onDragMove?: Handler;
  /** It was put down: `key` its data, `value` the target's id, "" when cancelled (Escape, `back`, or let go over no target). */
  onDragEnd?: Handler;
  /** A carried node was dropped on this target: `key` that node's data, `value` this target's box. */
  onDrop?: Handler;
  /** An on-screen stick: its box feeds this input action's x and y (from its centre, y up), with no handler. */
  touchStick?: string | undefined;
  /** An on-screen d-pad: its box feeds this input action, the nearest of eight directions. */
  touchPad?: string | undefined;
  /** An on-screen button: its box presses this input action while a touch that started on it is held. */
  touchButton?: string | undefined;
  /** A `touchStick`'s dead zone, a ratio of its radius from 0 to 0.95; 0 is the default 0.15. */
  deadZone?: number | undefined;
  /** Input actions handled where focus is (`{ confirm, details }`); the event's `key` names the action. */
  onAction?: Record<string, Handler> | undefined;
  disabled?: boolean | undefined;
  tabIndex?: number | undefined;
  /** The binding glyph of this input action (a Text's content): the pack's icon for it on the last-used device, an ink mask one em square, else its name (`E`, `LB`). */
  glyph?: string | undefined;
  /** crates/og-ui-next/CONVENTIONS.md §Accessibility. */
  role?: string | undefined;
  label?: string | undefined;
  description?: string | undefined;
  checked?: boolean | "mixed" | undefined;
  pressed?: boolean | "mixed" | undefined;
  selected?: boolean | undefined;
  expanded?: boolean | undefined;
  busy?: boolean | undefined;
  current?: boolean | undefined;
  hidden?: boolean | undefined;
  modal?: boolean | undefined;
  level?: number | undefined;
  live?: "polite" | "assertive" | "off" | undefined;
  valueMin?: number | undefined;
  valueMax?: number | undefined;
  valueNow?: number | undefined;
  valueText?: string | undefined;
  /**
   * The radial wipe of the box's image (an `Image`'s `src`, or a pack's
   * `nine-<name>` class): the share shown, 0 to 1 (1 whole), bound per tick;
   * from `wipeStart` radians (0 up), clockwise unless `wipeCcw`.
   */
  wipe?: number | undefined;
  wipeStart?: number | undefined;
  wipeCcw?: boolean | undefined;
  /**
   * Sits on an entity or a world point `[x, y, z]` instead of in its
   * parent's flow: its box's top left at that point as the tick's camera
   * sees it, at its laid-out size (move it with `translate-*` classes, such
   * as `-translate-x-1/2 -translate-y-full` to stand over it). Off the
   * screen, behind the camera or on a dead entity it is hidden: it paints
   * nothing, takes no press and leaves the accessibility tree. Never seen
   * by the simulation.
   */
  anchor?: Entity | readonly [number, number, number] | null | undefined;
  /** The anchor's offset in world units, such as `[0, 2, 0]` over a head. */
  lift?: readonly [number, number, number] | undefined;
  /** `data-*` states class variants read (`data-ready:`). */
  [data: `data-${string}`]: string | number | boolean | undefined;
}
