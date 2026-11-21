import { Icon, Show, Text, type UiEvent, View, t } from "@origingame/engine/ui";
import { NodeGraph } from "../components/panels/node-graph";
import { Button } from "../controls/button";
import "./flow-chart.en.ogt";

export interface FilmFlowchartScreenProps {
  /** og.film.chart's nodes as NodeGraph's: keyed rows of { id (the node), x, y, label, icon, state, shape, team }. state: current (where the film is), done (a segment seen), locked (not seen); an ending's shape diamond, a chapter's square, the rest circle. */
  nodes: ReadonlyArray<{ id: string; x: number; y: number; label: string; icon: string; state: string; shape: string; team: string }>;
  /** og.film.chart's links as NodeGraph's: keyed rows of { id, points, active }, points the link's FilmChartLink.points, active a choice the player has taken. */
  links: ReadonlyArray<{ id: string; points: readonly number[]; active: boolean }>;
  /** The chart's extent, [0, 0, chart.width, chart.height] from og.film.chart. */
  view: readonly [number, number, number, number];
  /** NodeGraph's zoom, kept by the game from its zoom and pinch actions. */
  zoom: number;
  /** NodeGraph's pan across. */
  panX: number;
  /** NodeGraph's pan down. */
  panY: number;
  /** The node picked (its id); empty is none. */
  selected: string;
  /** What the picked node is, as the game words it (its chapter, or Not seen yet); empty hides the panel. */
  detail: string;
  /** The picked node can be played from (a chapter the player has saved at). */
  canPlay: boolean;
  /** A node was picked: its id. */
  onPick?: (e: UiEvent, id: string) => void;
  /** The wheel over the chart: `delta` its notches, up negative (the game keeps zoom). */
  onZoom?: (e: UiEvent, delta: number) => void;
  /** The chart was dragged by `dx`, `dy` px (the game keeps the pan). */
  onPan?: (e: UiEvent, dx: number, dy: number) => void;
  /** Play from the picked node. */
  onPlay?: (e: UiEvent) => void;
  onBack?: (e: UiEvent) => void;
}

/**
 * The story so far over the dimmed film: a band with Back and the title,
 * the chart of film nodes (seen ones done, the one playing current, the
 * rest locked), and the picked node on a plate lit in the accent, its words
 * beside Play from here, which drops under them where the plate is narrow.
 */
export function FilmFlowchartScreen(props: FilmFlowchartScreenProps) {
  return (
    <View class="pt-8 pb-6 px-12 gap-y-4 text-accent-hover overflow-hidden compact:pt-4 compact:px-4 compact:gap-y-3">
      <View class="absolute bg-linear-to-b from-black/90 via-black/82 via-40% to-black/94 inset-0" />
      <View class="items-baseline gap-x-4 p-0 compact:flex-wrap compact:gap-y-1">
        <Button class="text-accent-hover/74 compact:-ml-2" onPress={(e) => props.onBack?.(e)}>
          <View class="items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
        <Text class="font-display text-[2.857rem] font-bold leading-tight tracking-xs text-accent-hover text-shadow-xs text-shadow-bg/60 grow shrink m-0 compact:text-[1.857rem] compact:basis-full">
          {t("screen.film-flowchart.title")}
        </Text>
      </View>
      <NodeGraph
        nodes={props.nodes}
        links={props.links}
        view={props.view}
        zoom={props.zoom}
        panX={props.panX}
        panY={props.panY}
        selected={props.selected}
        name={t("screen.film-flowchart.title")}
        onPick={(e, id) => props.onPick?.(e, id)}
        onZoom={(e, delta) => props.onZoom?.(e, delta)}
        onPan={(e, dx, dy) => props.onPan?.(e, dx, dy)}
      />
      <Show when={props.detail !== ""}>
        <View class="flex-row flex-wrap items-center gap-x-4 gap-y-2 py-3 px-4 bg-plate-lo/90 ring-1 ring-accent shrink-0">
          <View class="grow shrink basis-50 min-w-0 gap-y-0.5">
            <Text class="text-sm font-bold uppercase tracking-wide text-accent">{t("screen.film-flowchart.selected")}</Text>
            <Text class="text-xl font-semibold leading-snug text-fg">{props.detail}</Text>
          </View>
          <Button variant="primary" class="ml-auto min-w-45 compact:basis-full" disabled={!props.canPlay} onPress={(e) => { if (props.canPlay) props.onPlay?.(e); }}>
            <Text>{t("screen.film-flowchart.play")}</Text>
          </Button>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<FilmFlowchartScreenProps>> = {
  default: {"view":[0,0,720,360],"zoom":1,"panX":0,"panY":0,"selected":"hall","detail":"Chapter 2 · The hall","canPlay":true,"links":[{"id":"a","points":[60,60,93.2,60,119.4,60,140.9,60,160,60,179.1,60,200.6,60,226.8,60,260,60],"active":true},{"id":"b","points":[60,60,93.2,65.2,119.4,78.8,140.9,98,160,120,179.1,142,200.6,161.2,226.8,174.8,260,180],"active":false},{"id":"c","points":[260,60,293.2,60,319.4,60,340.9,60,360,60,379.1,60,400.6,60,426.8,60,460,60],"active":true},{"id":"d","points":[260,60,293.2,65.2,319.4,78.8,340.9,98,360,120,379.1,142,400.6,161.2,426.8,174.8,460,180],"active":false},{"id":"e","points":[460,60,493.2,60,519.4,60,540.9,60,560,60,579.1,60,600.6,60,626.8,60,660,60],"active":false}],"nodes":[{"id":"start","x":60,"y":60,"label":"Kitchen","icon":"flag","state":"done","shape":"square","team":""},{"id":"hall","x":260,"y":60,"label":"Hall","icon":"play","state":"done","shape":"square","team":""},{"id":"wait","x":260,"y":180,"label":"Wait","icon":"clock","state":"locked","shape":"circle","team":""},{"id":"brave","x":460,"y":60,"label":"Stairs","icon":"play","state":"current","shape":"circle","team":""},{"id":"shy","x":460,"y":180,"label":"Cellar","icon":"lock","state":"locked","shape":"circle","team":""},{"id":"end","x":660,"y":60,"label":"Dawn","icon":"crown","state":"locked","shape":"diamond","team":""}]},
  unseen: {"view":[0,0,720,360],"zoom":1,"panX":0,"panY":0,"selected":"shy","detail":"Not seen yet","canPlay":false,"links":[],"nodes":[]},
  none: {"view":[0,0,720,360],"zoom":1,"panX":0,"panY":0,"selected":"","detail":"","canPlay":false,"links":[],"nodes":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"pick":"Pick a node","play":"Play from here","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"film-flowchart","layer":"menu","focus":"Body/Graph","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"pop"} as const;
