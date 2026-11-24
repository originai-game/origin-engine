import { For, Show, View, packClass } from "@origingame/engine/ui";
import type { HudRegion } from "../../screens/hud-editor";

export interface HudLayoutProps {
  /** The regions as the player placed them (the HudEditorScreen's output). */
  regions?: ReadonlyArray<HudRegion> | undefined;
  /** Draws the HUD element a region holds; a region with no element draws nothing. */
  render?: ((region: HudRegion) => JSX.Element) | undefined;
  /** Classes after its own. */
  class?: string | undefined;
}

export interface HudPlaceProps {
  /** The region to place the children in. */
  region: HudRegion;
  /** Classes after its own. */
  class?: string | undefined;
  children?: any;
}

const clamp01 = (n: number) => Math.min(1, Math.max(0, n));

/**
 * One HUD element placed where a region says: its centre at the region's
 * `x`, `y` across the safe area, at its `scale` and `opacity`, absent while
 * `hidden`. Dressed by `HudLayout.region`.
 */
export function HudPlace(props: HudPlaceProps) {
  return (
    <Show when={props.region.hidden !== true}>
      <View
        class={`absolute flex-col items-center justify-center ${packClass("HudLayout", "region")} ${props.class ?? ""}`}
        data-region={props.region.id}
        style={{
          left: `${clamp01(props.region.x) * 100}%`,
          top: `${clamp01(props.region.y) * 100}%`,
          width: props.region.width ? `${clamp01(props.region.width) * 100}%` : undefined,
          height: props.region.height ? `${clamp01(props.region.height) * 100}%` : undefined,
          transform: `translate(-50%, -50%) scale(${props.region.scale ?? 1})`,
          opacity: props.region.opacity ?? 1,
        }}
      >
        {props.children}
      </View>
    </Show>
  );
}

/**
 * The gameplay HUD laid out from the regions a player saved in the
 * HudEditorScreen. The stage is the screen's safe area (`env(hud-inset-*)`
 * plus a margin), the same frame the editor shows, so a region sits where
 * the player put it. It takes no input itself; its elements keep theirs.
 */
export function HudLayout(props: HudLayoutProps) {
  return (
    <View
      class={`absolute inset-0 pointer-events-none pt-[calc(env(hud-inset-top)+1rem)] pr-[calc(env(hud-inset-right)+1rem)] pb-[calc(env(hud-inset-bottom)+1rem)] pl-[calc(env(hud-inset-left)+1rem)] ${props.class ?? ""}`}
    >
      <View class="relative size-full">
        <For each={props.regions ?? []} key={(r) => r.id}>
          {(r) => (
            <HudPlace region={r()}>
              <View class="pointer-events-auto">{props.render?.(r())}</View>
            </HudPlace>
          )}
        </For>
      </View>
    </View>
  );
}

/** The regions as text for a save or a settings file: a JSON array of the fields that matter. */
export function serializeHudLayout(regions: ReadonlyArray<HudRegion>): string {
  return JSON.stringify(
    regions.map((r) => ({
      id: r.id,
      label: r.label,
      x: r.x,
      y: r.y,
      ...(r.width !== undefined ? { width: r.width } : {}),
      ...(r.height !== undefined ? { height: r.height } : {}),
      ...(r.scale !== undefined ? { scale: r.scale } : {}),
      ...(r.opacity !== undefined ? { opacity: r.opacity } : {}),
      ...(r.hidden ? { hidden: true } : {}),
    })),
  );
}

/**
 * Regions read back from `serializeHudLayout` text, merged over `defaults`:
 * a saved region whose id is no longer in `defaults` is dropped, and a
 * default the save lacks keeps its default placement. Malformed text returns
 * `defaults`.
 */
export function deserializeHudLayout(text: string, defaults: ReadonlyArray<HudRegion>): HudRegion[] {
  let saved: unknown;
  try {
    saved = JSON.parse(text);
  } catch {
    return [...defaults];
  }
  if (!Array.isArray(saved)) return [...defaults];
  const byId = new Map<string, Record<string, unknown>>();
  for (const entry of saved) {
    if (entry && typeof entry === "object" && typeof (entry as { id?: unknown }).id === "string") {
      byId.set((entry as { id: string }).id, entry as Record<string, unknown>);
    }
  }
  const num = (v: unknown, fallback: number | undefined) => (typeof v === "number" && Number.isFinite(v) ? v : fallback);
  return defaults.map((d) => {
    const s = byId.get(d.id);
    if (!s) return d;
    return {
      ...d,
      x: clamp01(num(s.x, d.x) ?? d.x),
      y: clamp01(num(s.y, d.y) ?? d.y),
      width: num(s.width, d.width),
      height: num(s.height, d.height),
      scale: num(s.scale, d.scale),
      opacity: num(s.opacity, d.opacity),
      hidden: typeof s.hidden === "boolean" ? s.hidden : d.hidden,
    };
  });
}
