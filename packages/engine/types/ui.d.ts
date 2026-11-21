// @origingame/engine/ui as a game's type check reads it: the declarations of
// packages/engine/src/ui/index.ts, which src/ui/declared.ts holds it to (the
// engine's typecheck fails when the two differ). A game's check never follows
// the engine's source, which reads host APIs its lib does not declare.
/// <reference path="./assets.d.ts" />

import type { Texture, World } from "@origingame/engine";

declare global {
  namespace JSX {
    type Element = import("@origingame/engine/ui").UiChild;
    // Tags are PascalCase imports: no intrinsic elements.
    interface IntrinsicElements {}
    interface ElementChildrenAttribute { children: {} }
  }
}

/** Reads a value and, inside a computation, subscribes it. */
export type Accessor<T> = () => T;
/** A written value: read it by calling it, write it with `set`. */
export interface Signal<T> extends Accessor<T> {
  set(value: T): void;
  update(fn: (value: T) => T): void;
}
/** A value components below a provider read with `useContext`. */
export interface Context<T> { readonly id: symbol; readonly defaultValue: T }

export function createSignal<T>(value: T, options?: { equals?: false | ((a: T, b: T) => boolean) }): Signal<T>;
export function createMemo<T>(fn: (prev: T | undefined) => T, initial?: T, options?: { equals?: false | ((a: T, b: T) => boolean) }): Accessor<T>;
export function createRenderEffect<T>(fn: (prev: T | undefined) => T, initial?: T): void;
export function createEffect<T>(fn: (prev: T | undefined) => T, initial?: T): void;
export function untrack<T>(fn: () => T): T;
export function batch<T>(fn: () => T): T;
export function createRoot<T>(fn: (dispose: () => void) => T): T;
export function onCleanup(fn: () => void): void;
export function createContext<T>(defaultValue: T): Context<T>;
export function useContext<T>(context: Context<T>): T;

/** A node the renderer made: a handle on the UI tree's node `id`. */
export interface UiNode { readonly id: number }
/** What a child expression may give: nodes, text, nothing, lists and functions of them. */
export type UiChild = UiNode | string | number | boolean | null | undefined | UiChild[] | (() => UiChild);
/** What an event handler is given. */
export interface UiEvent {
  /** The world the UI belongs to, for writing intents and commands. */
  readonly world: World;
  readonly type: string;
  /** The pointer on the UI's surface. */
  readonly x: number;
  readonly y: number;
  /** A key's name (`onKey`) or the input action (`onAction`). */
  readonly key: string;
  /** A control's new value (`onChange`); a hold's progress, 0 to 1, "0" when let go early (`onHold`). */
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
  /** A finger held still on it for 0.5 s. */
  onLongPress?: Handler;
  /** A held press, with `hold` set: its progress each frame in `value`, 0 to 1 when it completes; "0" when it lets go early. */
  onHold?: Handler;
  /** How long a press must be held for `onHold` to complete, in seconds. */
  hold?: number | undefined;
  /** Takes focus when it appears (a dialog's first control). */
  autofocus?: boolean | undefined;
  /** Input actions handled where focus is (`{ confirm, details }`); the event's `key` names the action. */
  onAction?: Record<string, Handler> | undefined;
  disabled?: boolean | undefined;
  tabIndex?: number | undefined;
  /** The binding glyph of this input action (a Text's content). */
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
  /** `data-*` states class variants read (`data-ready:`). */
  [data: `data-${string}`]: string | number | boolean | undefined;
}

export const View: (props: HostProps) => UiNode;
export const Text: (props: HostProps) => UiNode;
/** A texture (`og.load.texture`; 0 none) drawn over its box, stretched; a class's `nine-<name>` slices a pack's. */
export const Image: (props: HostProps & { src?: Texture | 0 | undefined }) => UiNode;
export const Icon: (props: HostProps & { name?: string | undefined }) => UiNode;
export const Scroll: (props: HostProps) => UiNode;
/**
 * A polyline in its box: `points` are x, y pairs in CSS px from the box's
 * top left, stroked by `stroke-*` classes (colour and width; the ink and 1
 * px without), round joins and caps, `closed` back to the first point. No
 * fill, curves or dashes: what can be an image is an `Image`.
 */
export const Line: (props: HostProps & { points: readonly number[]; closed?: boolean | undefined }) => UiNode;
export const Input: (props: HostProps & { value?: string | undefined; placeholder?: string | undefined }) => UiNode;

/** `<Show when={c} fallback={…} keyed>`. */
export function Show<T>(props: { when: T | undefined | null | false; fallback?: UiChild; keyed?: boolean; children: UiChild | ((value: () => NonNullable<T>) => UiChild) }): UiChild;
/** `<For each={items} key={(i) => i.id}>{(item, index) => …}</For>`. */
export function For<T>(props: { each: readonly T[] | undefined | null; key?: (item: T) => unknown; children: (item: () => T, index: () => number) => UiChild }): UiChild;
/** One case of a `Switch`: shown when `when` is truthy and no case before it is. */
export function Match<T>(props: { when: T | undefined | null | false; children: UiChild | ((value: () => NonNullable<T>) => UiChild) }): UiChild;
/** The first case whose `when` holds, made again only when which case holds changes; `fallback` when none does. */
export function Switch(props: { fallback?: UiChild; children: UiChild }): UiChild;

/** The pack's classes for a component's slot (and its variant): put it last in the slot's `class`. */
export function packClass(component: string, slot: string, variant?: string | undefined): string;

/** The world of the mount being built or run. */
export function useWorld(): World;
/** The UI layers a mount may be on. */
export type UiLayer = "hud" | "menu" | "modal" | "popover" | "toast" | "tooltip" | "system";
/** Puts `Component(props)` on `world`'s UI, on `layer` (`hud` without one); returns the unmount. */
export function mount<P>(world: World, Component: (props: P) => UiChild, props: P, options?: { layer?: UiLayer }): () => void;
/** A store as a signal accessor, subscribed while the component lives. */
export function useStore<T>(store: { subscribe(run: (value: T) => void): () => void }): Accessor<T>;

/** The translation of `key` in the mount's locale (`params` fill its slots). */
export function t(key: string, params?: Record<string, unknown>): string;
export function formatNumber(value: number, format?: unknown): string;
export function formatCompact(value: number, decimals?: number): string;
export function formatDuration(seconds: number, style?: string): string;
export function formatCurrency(amount: number, currency: string): string;
