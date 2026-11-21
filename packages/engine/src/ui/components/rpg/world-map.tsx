import type { Texture } from "@origingame/engine";
import { For, Image, Line, Match, packClass, Show, Switch, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgWorldMapProps {
  /** The map's texture (og.load.texture); markers sit on it in fractions of its size. */
  map: Texture | 0;
  /** Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map. */
  markers?: ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string }>;
  /** The marker selected; empty is none. */
  selected: string;
  /** How far the map is zoomed in: 1 (and 0, and when left out) fits it to the box, 2 shows it twice the size. The game keeps it, from the zoom and pinch events, within its own limits. */
  zoom?: number;
  /** How far the zoomed map is moved across, CSS px (0, and when left out, pins its start corner). The game keeps it, from grab, pan and release. */
  panX?: number;
  /** How far the zoomed map is moved down, CSS px; 0 when left out. */
  panY?: number;
  /** The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it. */
  legend?: ReadonlyArray<{ id: string; kind: string; label: string }>;
  /** A caption pinned to the map's top corner (Current location); empty hides it. */
  here: string;
  /** The wheel zooms: the game reads the wheel's delta and keeps the zoom within its limits. */
  onZoom?: (e: UiEvent) => void;
  /** A press on the map begins a grab. */
  onGrab?: (e: UiEvent) => void;
  /** A grabbed map's moves pan it, each with x and y in the map's box. */
  onPan?: (e: UiEvent) => void;
  /** The grab's end releases it. */
  onRelease?: (e: UiEvent) => void;
  /** Choose a marker; the marker's key. The player's marker is not a button target. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/** A legend mark draws its marker's shape: a gold diamond, a red triangle, a cyan ring, a cyan disc. */
function LegendMark(props: { kind: string }) {
  return (
    <Switch fallback={
      <Show when={props.kind === "player"}>
        <View class={`shrink size-[1rem] rounded-full bg-[#38e8ff]`} />
      </Show>
    }>
      <Match when={props.kind === "quest"}>
        <View class={`shrink size-[0.714rem] rotate-45 bg-gold`} />
      </Match>
      <Match when={props.kind === "danger"}>
        <Line class={`shrink size-[1rem] stroke-[#ff4d5e]`} points={[7, 1, 13, 12, 1, 12]} closed />
      </Match>
      <Match when={props.kind === "waypoint"}>
        <View class={`shrink size-[1rem] rounded-full border-[3px] border-[#38e8ff]`} />
      </Match>
    </Switch>
  );
}

/**
 * A map with its markers, a one-line legend and an optional caption. The
 * map's art is the screen: no panel lies over it, only each marker's name
 * on a small cut plate, the legend and the caption. A marker's kind is its
 * shape as well as its colour (a quest a gold diamond, a waypoint a cyan
 * ring, danger a red triangle, the player a cyan disc with an arrow); its
 * name is white at any kind. The chosen marker is ringed and larger.
 * The map is drawn to cover the box it is given, times zoom and moved by
 * panX and panY; markers sit at fractions of it, keep their size at any
 * zoom and report choose (a marker's key). The player's marker is not a
 * button target. The game owns the zoom and the pan: the map reports zoom
 * (the wheel, with deltaY), pinch (two touches, with scale), and grab,
 * pan and release (a press, its moves and its end, each with x and y in
 * the map's box), and draws what the game sets.
 */
export function RpgWorldMap(props: RpgWorldMapProps) {
  return (
    <View class={`relative flex-col size-full text-fg`} label={t("rpg.map.name")}>
      <View
        class={`relative grow min-h-0 w-full overflow-hidden`}
        onWheel={(e) => props.onZoom?.(e)}
        onPointerDown={(e) => props.onGrab?.(e)}
        onPointerMove={(e) => props.onPan?.(e)}
        onPointerUp={(e) => props.onRelease?.(e)}
      >
        <View class="absolute left-0 top-0" style={{ width: `${((props.zoom ?? 1) || 1) * 100}%`, height: `${((props.zoom ?? 1) || 1) * 100}%`, translate: `${props.panX ?? 0}px ${props.panY ?? 0}px` }}>
          <Image class={`absolute inset-0 size-full`} src={props.map} />
          <View class="absolute inset-0" role="group" label={t("rpg.map.places")}>
            <For each={props.markers ?? []} key={(m) => m.id}>
              {(m) => (
                <View
                  class={`group/marker absolute size-11 -translate-x-1/2 -translate-y-1/2 overflow-visible flex-col items-center data-selected:z-10 disabled:opacity-100`}
                  style={{ left: `${m().x * 100}%`, top: `${m().y * 100}%` }}
                  role="button"
                  label={m().label}
                  disabled={m().kind === "player"}
                  selected={m().id === props.selected}
                  data-selected={m().id === props.selected}
                  onPress={(e) => props.onChoose?.(e, m().id)}
                >
                  <Show when={m().id === props.selected}>
                    <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[4.571rem] rounded-full shadow-[0px_0px_0px_2.288px_#ffffffeb,0px_0px_0px_6.857px_#ffcf5a59,0px_0px_22.857px_#ffcf5ab3]" />
                  </Show>
                  <View class="relative flex-row items-center justify-center size-[2.143rem] group-data-selected/marker:size-[2.857rem]">
                    <Switch fallback={
                      <Show when={m().kind === "player"}>
                        <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[71%] rounded-full bg-[#38e8ff] shadow-[0px_0px_0px_5px_#38e8ff47,0px_0px_0px_11px_#38e8ff1f,0px_0px_20px_#38e8ff]" />
                        <Line class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[60%] rotate-[35deg] stroke-[#0b0f18]" points={[8, 1, 14, 14, 8, 10.5, 2, 14]} closed />
                      </Show>
                    }>
                      <Match when={m().kind === "quest"}>
                        <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[62%] rotate-45 bg-linear-to-b from-[#ffe9ae] via-[#ffcf5a] to-[#c99b2e]" />
                        <View class="relative flex-col items-center gap-[0.128rem]">
                          <View class="w-[0.357rem] h-[0.929rem] bg-[#5c430d]" />
                          <View class="w-[0.357rem] h-[0.357rem] bg-[#5c430d]" />
                        </View>
                      </Match>
                      <Match when={m().kind === "danger"}>
                        <Line class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[2.143rem] stroke-[3px] stroke-[#ff4d5e]" points={[15, 3, 27, 26, 3, 26]} closed />
                        <View class="relative mt-[0.471rem] flex-col items-center gap-[0.128rem]">
                          <View class="w-[0.357rem] h-[0.929rem] bg-[#801823]" />
                          <View class="w-[0.357rem] h-[0.357rem] bg-[#801823]" />
                        </View>
                      </Match>
                      <Match when={m().kind === "waypoint"}>
                        <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[71%] rounded-full bg-[#38e8ff] shadow-[0px_0px_10px_#38e8ff99]" />
                        <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-[35%] rounded-full bg-[#0b1220]" />
                      </Match>
                    </Switch>
                  </View>
                  <Text class={`absolute left-1/2 -translate-x-1/2 top-[2.929rem] py-[0.143rem] px-1 whitespace-nowrap text-[1rem] font-semibold text-fg nine-plate-cut bg-[#060910e0] compact:hidden compact:group-data-selected/marker:block group-data-selected/marker:top-[3.286rem] group-data-selected/marker:text-[1.143rem] ${packClass("Chip", "frame")}`}>
                    {m().label}
                  </Text>
                </View>
              )}
            </For>
          </View>
        </View>
      </View>
      <Show when={props.here}>
        <View class={`absolute left-2 top-2 flex-row items-center gap-1 py-0.5 px-2 nine-plate-cut bg-[#1e2637e5] compact:left-1 compact:top-1 ${packClass("Plate", "frame")}`}>
          <View class={`shrink size-[1rem] rounded-full bg-[#38e8ff] shadow-[0px_0px_10px_#38e8ff99]`} />
          <Text class={`text-[0.929rem] text-fg whitespace-nowrap`}>{props.here}</Text>
        </View>
      </Show>
      <Show when={(props.legend ?? []).length}>
        <View class={`absolute left-2 bottom-2 max-w-[94%] flex-row flex-wrap items-center gap-x-2 gap-y-1 py-0.5 px-3 text-[0.929rem] nine-plate-cut bg-[#1e2637e5] compact:left-1 compact:right-1 compact:bottom-1 compact:flex-nowrap compact:justify-center compact:px-2 compact:text-[1rem] ${packClass("Plate", "frame")}`} role="group" label={t("rpg.map.legend")}>
          <For each={props.legend ?? []} key={(l) => l.id}>
            {(l) => (
              <View class={`shrink-0 flex-row items-center gap-1 whitespace-nowrap`}>
                <LegendMark kind={l().kind} />
                <Text class={`text-[0.929rem] text-fg compact:text-[1rem]`}>{l().label}</Text>
              </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgWorldMapProps>> = {
  default: {"map":0,"markers":[{"id":"crown","kind":"quest","x":0.48,"y":0.3,"label":"Dawnhold"},{"id":"me","kind":"player","x":0.3,"y":0.55,"label":"You"}],"selected":"crown","zoom":1,"panX":0,"panY":0,"legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"danger","kind":"danger","label":"Danger"}],"here":""},
  places: {"map":0,"markers":[{"id":"abbey","kind":"quest","x":0.42,"y":0.22,"label":"Saltmere Abbey"},{"id":"w1","kind":"waypoint","x":0.15,"y":0.4,"label":"Reedford"},{"id":"d1","kind":"danger","x":0.72,"y":0.32,"label":"Drowned Barrows"},{"id":"w2","kind":"waypoint","x":0.66,"y":0.62,"label":"Gullstone"},{"id":"me","kind":"player","x":0.34,"y":0.55,"label":"You"}],"selected":"abbey","zoom":1,"panX":0,"panY":0,"legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"waypoint","kind":"waypoint","label":"Waypoint"},{"id":"danger","kind":"danger","label":"Danger"},{"id":"player","kind":"player","label":"You"}],"here":"Brannoch Ford"},
};
