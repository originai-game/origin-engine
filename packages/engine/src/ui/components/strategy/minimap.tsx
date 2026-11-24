import type { Texture } from "@origingame/engine";
import { For, Image, packClass, Show, t, View } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyMinimapProps {
  /** The whole map's picture (og.load.texture), drawn square; 0 draws the dark well alone. */
  map: Texture | 0;
  /** What the minimap shows, keyed rows of { id, kind, x, y, base, label }: kind own, ally, foe or neutral (a resource, a relic; on data-kind, each its colour, a neutral a diamond); x and y on the map, 0 to 1; base marks a building or a town, drawn larger (data-base); label names it for a reader. */
  dots?: ReadonlyArray<{ id: string; kind: string; x: number; y: number; base: boolean; label: string }>;
  /** The camera's view on the map: its left edge, 0 to 1. */
  viewX: number;
  /** The camera's view: its top edge, 0 to 1. */
  viewY: number;
  /** The camera's view: its width, 0 to 1; 0 hides the frame. */
  viewWidth: number;
  /** The camera's view: its height, 0 to 1. */
  viewHeight: number;
  /** Alerts on the map, keyed rows of { id, x, y, label }: a red ring round the place (x and y, 0 to 1); label names it for a reader. */
  pings?: ReadonlyArray<{ id: string; x: number; y: number; label: string }>;
}

/**
 * The minimap at the bottom left: the whole map square in a lit-edged
 * tile with two corners cut, your units and buildings, your allies' and
 * your foes' as dots in their colours (a base larger, a resource a gold
 * diamond), the camera's view as a white frame and each alert a red ring.
 * The game gives every position on the map, 0 to 1; read only.
 */
export function StrategyMinimap(props: StrategyMinimapProps) {
  return (
    <View class={`shrink p-0.5 nine-plate-cut bg-sheen w-50 h-50 compact:w-24 compact:h-24 ${packClass("Plate", "frame")}`} role="group" label={t("strategy.minimap.name")}>
      <View class="relative size-full overflow-hidden nine-plate-cut bg-tile" role="image" label={t("strategy.minimap.name")}>
        <Show when={props.map}>
          <Image class="absolute inset-0 size-full" src={props.map} />
        </Show>
        <For each={props.dots ?? []} key={(d) => d.id}>
          {(d) => (
          <View class={`absolute size-1.25 -translate-x-1/2 -translate-y-1/2 bg-fg ring-regular ring-shade-deep data-base:size-2.25 data-own:bg-hud-accent data-ally:bg-info data-foe:bg-danger data-neutral:rotate-45 data-neutral:bg-gold compact:size-1 compact:data-base:size-2`} style={{ left: `${d().x * 100}%`, top: `${d().y * 100}%` }} role="image" label={d().label} data-base={d().base} data-own={d().kind === "own"} data-ally={d().kind === "ally"} data-foe={d().kind === "foe"} data-neutral={d().kind === "neutral"} />
          )}
        </For>
        <Show when={props.viewWidth > 0}>
          <View class="absolute ring ring-shade-deep inset-ring-regular inset-ring-knob" style={{ left: `${props.viewX * 100}%`, top: `${props.viewY * 100}%`, width: `${props.viewWidth * 100}%`, height: `${props.viewHeight * 100}%` }} />
        </Show>
        <For each={props.pings ?? []} key={(p) => p.id}>
          {(p) => (
          <View class="absolute size-4.5 -translate-x-1/2 -translate-y-1/2 rounded-full shadow-glow-sm shadow-foe inset-ring-regular inset-ring-foe compact:size-3.25" style={{ left: `${p().x * 100}%`, top: `${p().y * 100}%` }} role="image" label={p().label} />
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyMinimapProps>> = {
  default: {"map":0,"viewX":0.34,"viewY":0.38,"viewWidth":0.34,"viewHeight":0.24,"dots":[{"id":"keep","kind":"own","x":0.22,"y":0.5,"base":true,"label":"Your keep"},{"id":"u1","kind":"own","x":0.44,"y":0.52,"base":false,"label":"Men-at-Arms"},{"id":"u2","kind":"own","x":0.47,"y":0.51,"base":false,"label":"Archers"},{"id":"u3","kind":"own","x":0.5,"y":0.5,"base":false,"label":"Archers"},{"id":"u4","kind":"own","x":0.3,"y":0.36,"base":false,"label":"Villagers"},{"id":"camp","kind":"foe","x":0.82,"y":0.5,"base":true,"label":"Enemy camp"},{"id":"f1","kind":"foe","x":0.7,"y":0.46,"base":false,"label":"Raiders"},{"id":"f2","kind":"foe","x":0.73,"y":0.55,"base":false,"label":"Raiders"},{"id":"gold","kind":"neutral","x":0.2,"y":0.26,"base":false,"label":"Gold mine"}],"pings":[{"id":"p1","x":0.64,"y":0.34,"label":"The east tower is under attack"}]},
  allies: {"map":0,"viewX":0.05,"viewY":0.6,"viewWidth":0.3,"viewHeight":0.22,"dots":[{"id":"keep","kind":"own","x":0.18,"y":0.7,"base":true,"label":"Your keep"},{"id":"ally","kind":"ally","x":0.75,"y":0.25,"base":true,"label":"Ally keep"},{"id":"a1","kind":"ally","x":0.6,"y":0.35,"base":false,"label":"Ally army"},{"id":"relic","kind":"neutral","x":0.5,"y":0.5,"base":false,"label":"Relic"}],"pings":[]},
  empty: {"map":0,"viewX":0,"viewY":0,"viewWidth":0,"viewHeight":0,"dots":[],"pings":[]},
  overflow: {"map":0,"viewX":0.8,"viewY":0.85,"viewWidth":0.4,"viewHeight":0.3,"dots":[{"id":"d1","kind":"own","x":0,"y":0,"base":true,"label":"Corner"},{"id":"d2","kind":"foe","x":1,"y":1,"base":true,"label":"Corner"},{"id":"d3","kind":"ally","x":1,"y":0,"base":false,"label":"Corner"},{"id":"d4","kind":"neutral","x":0,"y":1,"base":false,"label":"Corner"},{"id":"d5","kind":"foe","x":0.5,"y":0.5,"base":false,"label":"Centre"},{"id":"d6","kind":"foe","x":0.51,"y":0.5,"base":false,"label":"Centre"},{"id":"d7","kind":"foe","x":0.52,"y":0.51,"base":false,"label":"Centre"}],"pings":[{"id":"p1","x":0.02,"y":0.98,"label":"A corner raid"},{"id":"p2","x":0.5,"y":0.5,"label":"The centre"},{"id":"p3","x":0.9,"y":0.1,"label":"The far north east"}]},
};
