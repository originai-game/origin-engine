import { For, Icon, Line, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./node-graph.en.ogt";

export interface NodeGraphProps {
  /** The nodes: each { id, x, y, label, icon, state, shape, team }, x and y their centres in graph units; state available, locked, active, done or current (a rim and a dim as well as a colour); shape circle, square, hex or diamond; team ally, enemy or empty (pair a side with a shape, so sides are not told by colour alone). */
  nodes: ReadonlyArray<{ id: string; x: number; y: number; label: string; icon: string; state: string; shape: string; team: string }>;
  /** The links under the nodes: each { id, points, active }, points x, y pairs in graph units (og.film.chart's link points, or the game's own curve sampled), active a path taken. */
  links: ReadonlyArray<{ id: string; points: readonly number[]; active: boolean }>;
  /** The graph's extent in graph units, [x, y, width, height]: it is fitted into the well whole, scaled uniformly and centred. */
  view: readonly [number, number, number, number];
  /** Its zoom about the middle; the game keeps it from onZoom. Default 1. */
  zoom?: number;
  /** How far it is panned across and down, px; the game keeps them from onPan. */
  panX?: number;
  panY?: number;
  /** The selected node's id: ringed in the focus colour. */
  selected?: string;
  /** What a screen reader calls it; empty is Graph. */
  name?: string;
  /** A node was pressed (a locked one too: the game says why it is closed). */
  onPick?: (e: UiEvent, id: string) => void;
  /** The wheel over the well: `delta` its notches, up negative. */
  onZoom?: (e: UiEvent, delta: number) => void;
  /** The well was dragged by `dx`, `dy` px since the last move. */
  onPan?: (e: UiEvent, dx: number, dy: number) => void;
}

/**
 * A graph on the HUD's smoked glass: links as lines (lit where taken) under
 * nodes on plates, each captioned beneath on a chip of the well. The view
 * is fitted into the well whole and centred; only where nodes stand and the
 * lines between them scale, a node keeps its size. Zoom and pan are the
 * game's, from the wheel and a drag. Reports pick (a node's id).
 */
export function NodeGraph(props: NodeGraphProps) {
  const v = () => props.view;
  // A drag's last pointer, so a move reports how far it went.
  let last: [number, number] | null = null;
  return (
    <View
      class="relative grow self-stretch min-h-[13.714rem] min-w-0 overflow-hidden bg-glass/88 border border-white/12"
      role="group"
      label={props.name || t("component.node-graph.name")}
      valueText={t("component.node-graph.zoom", { zoom: props.zoom ?? 1 })}
      onWheel={(e) => props.onZoom?.(e, Math.sign(e.y))}
      onPointerDown={(e) => { last = [e.x, e.y]; }}
      onPointerMove={(e) => {
        if (!last) return;
        const [x, y] = last;
        last = [e.x, e.y];
        props.onPan?.(e, e.x - x, e.y - y);
      }}
      onPointerUp={() => { last = null; }}
    >
      <View class="absolute inset-12 compact:inset-8 items-center justify-center">
        <View
          class="relative w-full max-h-full"
          style={{ "aspect-ratio": `${v()[2]} / ${v()[3]}`, translate: `${props.panX ?? 0}px ${props.panY ?? 0}px`, scale: `${props.zoom ?? 1}` }}
        >
          <For each={props.links} key={(l) => l.id}>
            {(l) => (
              <Line class="absolute inset-0 size-full stroke-[2px] stroke-white/25 data-active:stroke-hud-accent data-active:stroke-[3px]" points={l().points} viewBox={`${v()[0]} ${v()[1]} ${v()[2]} ${v()[3]}`} data-active={l().active} />
            )}
          </For>
          <For each={props.nodes} key={(n) => n.id}>
            {(n) => (
              <View
                class="group/node absolute size-0"
                style={{ left: `${((n().x - v()[0]) / v()[2]) * 100}%`, top: `${((n().y - v()[1]) / v()[3]) * 100}%` }}
                data-locked={n().state === "locked"}
                data-active={n().state === "active"}
                data-done={n().state === "done"}
                data-current={n().state === "current"}
                data-selected={n().id === props.selected}
                data-ringed={n().id === props.selected && n().shape !== "hex"}
                data-ally={n().team === "ally"}
                data-enemy={n().team === "enemy"}
                data-diamond={n().shape === "diamond"}
                data-square={n().shape === "square"}
                data-hex={n().shape === "hex"}
              >
                <View
                  class="absolute -ml-6 -mt-6 size-12 compact:-ml-4.5 compact:-mt-4.5 compact:size-9 items-center justify-center rounded-full bg-well border-2 border-white/30 text-fg group-data-square/node:rounded-md group-data-diamond/node:rounded-sm group-data-diamond/node:rotate-45 group-data-hex/node:rounded-none group-data-hex/node:bg-transparent group-data-hex/node:border-transparent group-data-ally/node:bg-info/45 group-data-enemy/node:bg-danger/45 group-data-done/node:border-success group-data-active/node:border-hud-accent group-data-current/node:border-hud-accent group-data-locked/node:text-fg-subtle group-data-ringed/node:border-focus group-data-ringed/node:border-[3px]"
                  role="button"
                  label={n().label}
                  valueText={n().state}
                  disabled={n().state === "locked"}
                  selected={n().id === props.selected}
                  onPress={(e) => props.onPick?.(e, n().id)}
                >
                  <Show when={n().shape === "hex"}>
                    <View class="absolute inset-0 nine-disc bg-well group-data-ally/node:bg-info/45 group-data-enemy/node:bg-danger/45" />
                    <Line class="absolute inset-0 size-full stroke-[2px] stroke-white/30 group-data-done/node:stroke-success group-data-active/node:stroke-hud-accent group-data-current/node:stroke-hud-accent group-data-selected/node:stroke-focus group-data-selected/node:stroke-[3px]" points={[50, 2, 92, 26, 92, 74, 50, 98, 8, 74, 8, 26]} closed viewBox={[100, 100]} />
                  </Show>
                  <Icon class="size-6 compact:size-4.5 group-data-diamond/node:-rotate-45 group-data-locked/node:opacity-45" name={n().icon} />
                </View>
                <Text class="absolute -left-24 w-48 top-7 compact:top-5.5 text-center whitespace-nowrap text-xs font-semibold text-fg group-data-locked/node:text-fg-subtle">
                  {n().label}
                </Text>
              </View>
            )}
          </For>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<NodeGraphProps>> = {
  default: {"view":[0,0,360,240],"zoom":1,"panX":0,"panY":0,"selected":"cleave","name":"","links":[{"id":"a","points":[40,120,70,118,100,108,130,86,155,68,180,60],"active":true},{"id":"b","points":[40,120,70,122,100,132,130,154,155,172,180,180],"active":false},{"id":"c","points":[180,60,320,120],"active":false}],"nodes":[{"id":"root","x":40,"y":120,"label":"Strength","icon":"sword","state":"done","shape":"circle","team":""},{"id":"cleave","x":180,"y":60,"label":"Cleave","icon":"bolt","state":"available","shape":"hex","team":""},{"id":"guard","x":180,"y":180,"label":"Guard","icon":"shield","state":"active","shape":"square","team":"ally"},{"id":"rage","x":320,"y":120,"label":"Rage","icon":"fire","state":"locked","shape":"diamond","team":"enemy"}]},
  zoomed: {"view":[0,0,360,240],"zoom":1.6,"panX":-120,"panY":40,"selected":"","name":"","links":[{"id":"a","points":[40,120,180,60],"active":true},{"id":"c","points":[180,60,320,120],"active":false}],"nodes":[{"id":"root","x":40,"y":120,"label":"Strength","icon":"sword","state":"current","shape":"circle","team":""},{"id":"cleave","x":180,"y":60,"label":"Cleave","icon":"bolt","state":"available","shape":"circle","team":""},{"id":"rage","x":320,"y":120,"label":"Rage","icon":"fire","state":"locked","shape":"circle","team":""}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"pick":"Pick a node","zoom":"Zoom (wheel)","pan":"Pan"} as const;
