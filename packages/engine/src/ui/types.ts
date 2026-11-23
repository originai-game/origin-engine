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
  /** With `focusGroup`: the group is one Tab stop (a tab list, a radio group, a toolbar). Tab enters it at its focused member, else its selected, checked or pressed one, else the one it last had, else its first; the arrows move among them all. */
  focusRoving?: boolean | undefined;
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
  /** Two fingers that went down on it pinch: `key` is "start", "move" or "end", `value` the scale since they began, x and y their midpoint. */
  onPinch?: Handler;
  /** Two fingers that went down on it turn: as `onPinch`, `value` the turn since they began in radians, clockwise on screen. */
  onRotate?: Handler;
  /** A pointer that went down on it was let go moving fast (300 px/s, 10 px): `key` its way, "left", "right", "up" or "down"; `value` its velocity, "vx vy" in px/s. A pinch's fingers swipe nothing. */
  onSwipe?: Handler;
  /** An on-screen stick: its box feeds this input action's x and y (from its centre, y up), with no handler. Like `touchPad` and `touchButton`, it is `active:` while a touch holds it. */
  touchStick?: string | undefined;
  /** An on-screen d-pad: its box feeds this input action, the nearest of eight directions. Held, it carries the way it reads as `data-dir-up`, `data-dir-right`, `data-dir-down` and `data-dir-left` (two on a diagonal), so a class lights the pressed arm (`group-data-dir-up/pad:`). */
  touchPad?: string | undefined;
  /** An on-screen button: its box presses this input action while a touch that started on it is held. */
  touchButton?: string | undefined;
  /** A `touchStick`'s dead zone, a ratio of its radius from 0 to 0.95; 0 is the default 0.15. */
  deadZone?: number | undefined;
  /** Inside a `touchStick`: it moves with the stick's knob as og-input reads it. */
  knob?: boolean | undefined;
  /** Input actions handled where focus is (`{ confirm, details }`); the event's `key` names the action. */
  onAction?: Record<string, Handler> | undefined;
  disabled?: boolean | undefined;
  tabIndex?: number | undefined;
  /** The binding glyph of this input action (a Text's content): the pack's icon for it on the last-used device, an ink mask one em square, else its name (`E`, `LB`). */
  glyph?: string | undefined;
  /**
   * The pack event this node sounds when it first paints, once: the sound
   * its pack slot has for it (the slot `packClass` names in its class),
   * else the pack's own, and none when the pack has neither; an event the
   * pack has no clip for sounds the nearest it has (toggle and tab:
   * select, purchase: confirm, levelup: reward, countdown: press). These
   * are the events no input fires; a control's hover, press, select,
   * confirm, cancel and back sound by themselves.
   */
  sound?: "open" | "close" | "error" | "reward" | "toggle" | "tab" | "levelup" | "purchase" | "countdown" | undefined;
  /** crates/og-ui-next/CONVENTIONS.md §Accessibility. */
  role?: string | undefined;
  label?: string | undefined;
  description?: string | undefined;
  checked?: boolean | "mixed" | undefined;
  pressed?: boolean | "mixed" | undefined;
  selected?: boolean | undefined;
  expanded?: boolean | undefined;
  busy?: boolean | undefined;
  /** Its value fails the game's checks: screen readers say invalid, and `aria-invalid:` classes apply. */
  invalid?: boolean | undefined;
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
   * screen, behind the camera or on a dead entity it is hidden (unless
   * `clamp`): it paints nothing, takes no press and leaves the
   * accessibility tree. Never seen by the simulation.
   *
   * A string anchors it to the UI node of that `name` instead: its top
   * left at that node's laid-out box's top left, `lift`'s x and y fractions
   * of that box's size (`lift={[0, 1, 0]}` stands under it), following it
   * every frame; hidden while no node of that name shows. `occlusion`,
   * `declutter` and `clamp` are for world anchors.
   */
  anchor?: Entity | readonly [number, number, number] | string | null | undefined;
  /** The anchor's offset in world units, such as `[0, 2, 0]` over a head. */
  lift?: readonly [number, number, number] | undefined;
  /**
   * Behind a drawn mesh, as the tick's camera sees the anchor's point
   * (other than its own entity's): `fade` draws it at 0.35 opacity, `hide`
   * hides it; it carries `data-occluded` either way. Checked at each
   * tick's end.
   */
  occlusion?: "none" | "fade" | "hide" | undefined;
  /**
   * Over another anchored element that declutters: `stack` moves it
   * straight up clear of it, `hide` hides it. Higher `priority`, then the
   * nearer, keeps its place.
   */
  declutter?: "none" | "stack" | "hide" | undefined;
  /** Which of two decluttering anchors keeps its place: higher first (0). */
  priority?: number | undefined;
  /**
   * `edge`: off the screen or behind the camera it stays, pinned
   * `clampMargin` px (24) inside the screen's edge toward its point, and
   * carries `data-clamped` and `data-edge-top|right|bottom|left`.
   */
  clamp?: "none" | "edge" | undefined;
  clampMargin?: number | undefined;
  /**
   * What `anchor` and `spotlight` find it by. Of the live nodes that
   * share a name, the last made answers. An `Icon`'s `name` is its icon:
   * name a `View` around it.
   */
  name?: string | undefined;
  /**
   * A tutorial's spotlight: the `name` of the node it lights. Its own
   * `bg-*` covers the whole surface outside that node's box, and it takes
   * every press there (whether it listens or not: nothing under it gets
   * one), so the lit node keeps its own presses. Its children paint as
   * laid out. While no node of that name shows, it is an ordinary box.
   * It carries `data-lit` while its node shows (`data-lit:bg-scrim`, and
   * `group-data-lit/<g>:hidden` for a fallback without a target).
   */
  spotlight?: string | undefined;
  /** `data-*` states class variants read (`data-ready:`). */
  [data: `data-${string}`]: string | number | boolean | undefined;
}
