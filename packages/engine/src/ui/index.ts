// @origingame/engine/ui: what a game's TSX imports (crates/og-ui-next/CONVENTIONS.md
// §Components). The primitives compile away (nsh-jsx makes their host elements
// in place); these definitions give them their types and serve a primitive used
// as a value. `mount` puts a component on a world's UI; `useStore` reads a store
// as a signal; `t` and the formatters read the mount's world.

import type { World } from "@origingame/engine";
import { abi } from "../runtime/host.ts";
import { json } from "../runtime/result.ts";
import { type Renderer, Renderer as UiRenderer, RendererContext, type UiChild, type UiEvent, UiNode, component, element, insert, spread } from "./rt.ts";
import { type Accessor, createContext, createMemo, createRoot, createSignal, onCleanup, provide, untrack, useContext } from "./signals.ts";

export { batch, createContext, createEffect, createMemo, createRenderEffect, createRoot, createSignal, onCleanup, untrack, useContext } from "./signals.ts";
export type { Accessor, Context, Signal } from "./signals.ts";
export type { UiChild, UiEvent, UiNode } from "./rt.ts";

declare global {
  namespace JSX {
    type Element = UiChild;
    // Tags are PascalCase imports: no intrinsic elements.
    interface IntrinsicElements {}
    interface ElementChildrenAttribute { children: {} }
  }
}

/** The id the host calls take: the runtime's world (WorldImpl) carries it. */
function idOf(world: World): number {
  return (world as unknown as { readonly id: number }).id;
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
  /** Input actions handled where focus is (`{ confirm, details }`); the event's `key` names the action. */
  onAction?: Record<string, Handler> | undefined;
  disabled?: boolean | undefined;
  tabIndex?: number | undefined;
  /** The binding glyph of this input action (a Text's content). */
  glyph?: string | undefined;
  /** CONVENTIONS.md §Accessibility. */
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
  /** `data-*` states class variants read (`data-ready:`). */
  [data: `data-${string}`]: string | number | boolean | undefined;
}

function host<P extends HostProps>(tag: string): (props: P) => UiNode {
  return (props) => {
    const node = element(tag);
    spread(node, props as unknown as Record<string, unknown>);
    if ("children" in props) insert(node, () => props.children);
    return node;
  };
}

export const View = host<HostProps>("view");
export const Text = host<HostProps>("text");
export const Image = host<HostProps & { src?: string | undefined }>("image");
export const Icon = host<HostProps & { name?: string | undefined }>("icon");
export const Scroll = host<HostProps>("scroll");
export const Input = host<HostProps & { value?: string | undefined; placeholder?: string | undefined }>("input");
export const Svg = host<HostProps & { viewBox?: string | undefined }>("svg");

/** `<Show when={c} fallback={…} keyed>`: compiled to the renderer's `cond`. */
export declare function Show<T>(props: { when: T | undefined | null | false; fallback?: UiChild; keyed?: boolean; children: UiChild | ((value: () => NonNullable<T>) => UiChild) }): UiChild;
/** `<For each={items} key={(i) => i.id}>{(item, index) => …}</For>`: compiled to the renderer's `each`. */
export declare function For<T>(props: { each: readonly T[] | undefined | null; key?: (item: T) => unknown; children: (item: () => T, index: () => number) => UiChild }): UiChild;

const MATCH = Symbol("Match");
type MatchProps<T> = { when: T | undefined | null | false; children: UiChild | ((value: () => NonNullable<T>) => UiChild) };

/** One case of a `Switch`: shown when `when` is truthy and no case before it is. */
export function Match<T>(props: MatchProps<T>): UiChild {
  return { [MATCH]: props } as unknown as UiChild;
}

/**
 * `<Switch fallback={…}><Match when={a}>…</Match><Match when={b}>…</Match></Switch>`:
 * the first case whose `when` holds, made again only when which case holds
 * changes; `fallback` when none does.
 */
export function Switch(props: { fallback?: UiChild; children: UiChild }): UiChild {
  const cases = createMemo(() => {
    const children = props.children;
    const out: MatchProps<unknown>[] = [];
    const walk = (c: unknown): void => {
      if (Array.isArray(c)) c.forEach(walk);
      else if (c && typeof c === "object" && MATCH in c) out.push((c as { [MATCH]: MatchProps<unknown> })[MATCH]);
    };
    walk(props.children);
    return out;
  });
  const index = createMemo(() => cases().findIndex((c) => Boolean(c.when)));
  return createMemo(() => {
    const i = index();
    if (i < 0) return props.fallback;
    const c = cases()[i]!;
    const content = c.children;
    return untrack(() => (typeof content === "function" && content.length > 0 ? content(() => c.when as never) : content));
  }) as unknown as UiChild;
}

/**
 * The pack's classes for a component's slot (and its variant): put it last
 * in the slot's `class`. It answers a `@Component.slot[.variant]` token that
 * the UI's class resolver replaces with the pack's strings; the cook folds a
 * call with literal arguments into the strings themselves.
 */
export function packClass(component: string, slot: string, variant?: string | undefined): string {
  return variant ? `@${component}.${slot}.${variant}` : `@${component}.${slot}`;
}

const WorldContext = createContext<World | null>(null);

/** The world of the mount being built or run. */
export function useWorld(): World {
  const world = useContext(WorldContext);
  if (!world) throw new Error("og.ui: useWorld is read inside a mount");
  return world;
}

/** Each world's renderer: its op buffer and handlers. */
export const renderers = new Map<number, Renderer>();

export function rendererFor(world: World): Renderer {
  let ui = renderers.get(idOf(world));
  if (!ui) {
    ui = new UiRenderer((bytes) => abi().og_engine_ui_patch(idOf(world), bytes), world);
    renderers.set(idOf(world), ui);
  }
  return ui;
}

/** The UI layers a mount may be on. */
export type UiLayer = "hud" | "menu" | "modal" | "popover" | "toast" | "tooltip" | "system";

/**
 * Puts `Component(props)` on `world`'s UI, on `layer` (`hud` without one), and
 * sends it at once, so it builds while the game loads. Returns the unmount.
 */
export function mount<P>(world: World, Component: (props: P) => UiChild, props: P, options?: { layer?: UiLayer }): () => void {
  const ui = rendererFor(world);
  let dispose!: () => void;
  const root = createRoot((d) => {
    dispose = d;
    return provide(RendererContext, ui, () => provide(WorldContext, world, () => {
      const node = element("view");
      ui.setProp(node, "layer", options?.layer ?? "hud");
      insert(node, component(Component, props));
      return node;
    }));
  });
  ui.insert(0, root, null);
  ui.flush();
  return () => {
    dispose();
    ui.remove(0, root);
    ui.flush();
  };
}

/** A store as a signal accessor, subscribed while the component lives. */
export function useStore<T>(store: { subscribe(run: (value: T) => void): () => void }): Accessor<T> {
  const value = createSignal<T>(undefined as T, { equals: false });
  onCleanup(store.subscribe((v) => value.set(v)));
  return value;
}

/** The translation of `key` in the mount's locale (`params` fill its slots). */
export function t(key: string, params?: Record<string, unknown>): string {
  return abi().og_i18n_t(idOf(useWorld()), key, json(params));
}
export function formatNumber(value: number, format?: unknown): string {
  return abi().og_i18n_format_number(idOf(useWorld()), value, json(format));
}
export function formatCompact(value: number, decimals?: number): string {
  return abi().og_i18n_format_compact(idOf(useWorld()), value, decimals);
}
export function formatDuration(seconds: number, style?: string): string {
  return abi().og_i18n_format_duration(idOf(useWorld()), seconds, style);
}
export function formatCurrency(amount: number, currency: string): string {
  return abi().og_i18n_format_currency(idOf(useWorld()), amount, currency);
}
