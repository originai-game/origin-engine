// @origingame/engine/ui: what a game's TSX imports (crates/og-ui-next/CONVENTIONS.md
// §Components). The primitives compile away (nsh-jsx makes their host elements
// in place); these definitions give them their types and serve a primitive used
// as a value. `mount` puts a component on a world's UI; `useStore` reads a store
// as a signal; `t` and the formatters read the mount's world.

import type { Texture, World } from "@origingame/engine";
import { abi } from "../runtime/host.ts";
import { json } from "../runtime/result.ts";
import { type Renderer, Renderer as UiRenderer, RendererContext, type UiNode, component, element, insert, spread } from "./rt.ts";
import { type Accessor, createContext, createMemo, createRoot, createSignal, onCleanup, provide, untrack, useContext } from "./signals.ts";

export { batch, createContext, createEffect, createMemo, createRenderEffect, createRoot, createSignal, onCleanup, untrack, useContext } from "./signals.ts";
export type { Accessor, Context, Signal } from "./signals.ts";
// The public shapes are the declaration's (types/ui.d.ts), which games read.
import type { HostProps, UiChild } from "../../types/ui.d.ts";
export type { HostProps, UiChild, UiEvent, UiNode } from "../../types/ui.d.ts";


/** The id the host calls take: the runtime's world (WorldImpl) carries it. */
function idOf(world: World): number {
  return (world as unknown as { readonly id: number }).id;
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
/** A texture (`og.load.texture`; 0 none) drawn over its box, stretched; a class's `nine-<name>` slices a pack's. */
export const Image = host<HostProps & { src?: Texture | 0 | undefined }>("image");
export const Icon = host<HostProps & { name?: string | undefined }>("icon");
export const Scroll = host<HostProps>("scroll");
/**
 * A polyline in its box: `points` are x, y pairs in CSS px from the box's
 * top left, stroked by `stroke-*` classes (colour and width; the ink and 1
 * px without), round joins and caps, `closed` back to the first point. With
 * `viewBox` (`[w, h]`, or SVG's `"x y w h"`) the points are in that space,
 * mapped onto the laid-out box as it paints: stretched to fill it, or with
 * `fit="contain"` scaled uniformly and centred; the stroke stays in px. No
 * fill, curves or dashes: what can be an image is an `Image`.
 */
export const Line = host<HostProps & { points: readonly number[]; closed?: boolean | undefined; viewBox?: readonly [number, number] | string | undefined; fit?: "fill" | "contain" | undefined }>("line");
export const Input = host<HostProps & { value?: string | undefined; placeholder?: string | undefined }>("input");

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
  ui.sendTables();
  let dispose!: () => void;
  const root = createRoot((d) => {
    dispose = d;
    return provide(RendererContext, ui, () => provide(WorldContext, world, () => {
      const node = element("view");
      // The mount fills the UI's surface: a component's root lays out in it
      // as in a full-screen column, and `absolute inset-0` covers the screen.
      ui.setClass(node, "absolute inset-0 flex-col");
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
