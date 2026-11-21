import { Show, Text, View } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface WaypointProps {
  /** What it marks, by its bits (0: use at). */
  entity?: number;
  /** A world point it marks without an entity (x y z). */
  at?: string;
  /** World units above the point. */
  height?: number;
  /** What it is: objective (empty, gold), ping (cyan, a ripple), danger (red), ally (green) or loot (violet). */
  kind?: string;
  /** Its name in the kind's ink (The Ninefold Gate); empty is none. It drops when far. */
  label?: string;
  /** How far it is as the game words it (412), on a dark tag; empty is none. */
  distance?: string;
  /** The distance's unit, smaller after it (m). */
  units?: string;
  /** near (a soft halo), mid (empty) or far (the label drops, the distance stays), as the game judges it. */
  range?: string;
  /** The side of the screen it is off, as the game reads it (left or right): a chevron toward it before the pin; empty is on screen. The anchor holds it inside the screen's edge. */
  edge?: string;
}

/**
 * A place to go: a lit diamond with a dark core in the kind's colour, its
 * name under it and the distance on a dark tag. Off screen, a chevron
 * points the way. Read only; the game owns the distance and what counts
 * as near and far. Off screen or behind the camera it pins 24 px inside
 * the screen's edge toward its point (the anchor's clamp, which marks the
 * side as `data-edge-*`). The old anchor's decluttering, avoidance and
 * chevron have no TSX surface: the edge prop leans the label toward the
 * side the game names, and the chevron it drew is dropped.
 */
export function Waypoint(props: WaypointProps) {
  const kind = () => ((props.kind ?? "") == "" ? "objective" : (props.kind ?? ""));
  const range = () => ((props.range ?? "") == "" ? "mid" : (props.range ?? ""));
  const edge = () => ((props.edge ?? "") == "" ? "none" : (props.edge ?? ""));
  const label = () => props.label ?? "";
  const distance = () => props.distance ?? "";
  const units = () => props.units ?? "";
  const point = () => ((props.at ?? "0 0 0").split(/\s+/).map(Number) as [number, number, number]);
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : point()}
      lift={[0, props.height ?? 0, 0]}
      clamp="edge"
      class="group flex-col items-center gap-y-[0.214rem]"
      label={label() !== "" ? label() : distance()}
      data-objective={kind() === "objective"}
      data-ping={kind() === "ping"}
      data-danger={kind() === "danger"}
      data-ally={kind() === "ally"}
      data-loot={kind() === "loot"}
      data-near={range() === "near"}
      data-far={range() === "far"}
      data-way-left={edge() === "left"}
      data-way-right={edge() === "right"}
    >
      <View class="relative m-1 size-[1.214rem] rotate-45 bg-gold shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0.857rem_#ffd23f] group-data-danger:bg-danger group-data-danger:shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0.857rem_#ff4d5e] group-data-ally:bg-success group-data-ally:shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0.857rem_#55e39a] group-data-loot:bg-epic group-data-loot:shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0.857rem_#c86ff5] group-data-ping:bg-rare group-data-ping:shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0.857rem_#5cc8ff] group-data-ping:animate-ping group-data-near:shadow-[0px_0px_0px_1.5px_#0d1016d9,0px_0px_0px_0.5rem_#ffd23f38,0px_0px_1.286rem_#ffd23f]">
        <View class="absolute inset-[32%] bg-tile/78" />
      </View>
      <Show when={(label() !== "" && range() !== "far") || (distance() !== "")}>
        <View class="flex-col items-center gap-y-[0.143rem] min-w-0 group-data-way-left:items-start group-data-way-right:items-end group-data-far:opacity-[0.82]">
          <Show when={label() !== "" && range() !== "far"}>
            <Text class="min-w-0 max-w-[14.286rem] text-[0.821rem] leading-[1.1] font-bold tracking-[0.13em] uppercase whitespace-nowrap truncate text-gold text-shadow-2xs text-stroke-3 text-stroke-bg group-data-danger:text-danger group-data-ally:text-success group-data-loot:text-epic group-data-ping:text-rare">{label()}</Text>
          </Show>
          <Show when={distance() !== ""}>
            <View class="shrink-0 flex-row items-baseline py-[0.071rem] px-[0.429rem] nine-chip-cut bg-raised-pressed/88">
              <Text class="text-caption leading-[1.2] font-bold tabular-nums whitespace-nowrap text-fg">{distance()}</Text>
              <Show when={units() !== ""}>
                <Text class="ml-[1px] text-[0.725rem] leading-[1.2] text-fg/66">{units()}</Text>
              </Show>
            </View>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<WaypointProps>> = {
  default: {"entity":0,"at":"0 0 0","height":1.5,"kind":"","label":"The Ninefold Gate","distance":"412","units":"m","range":"","edge":""},
  near: {"label":"Ninefold Gate","distance":"12","range":"near"},
  far: {"distance":"1,240","range":"far"},
  ping: {"kind":"ping","label":"Look here","distance":"34"},
  danger: {"kind":"danger","label":"Enemy","distance":"58"},
  ally: {"kind":"ally","label":"Sylvenne","distance":"22"},
  loot: {"kind":"loot","label":"Cache","distance":"140"},
  offscreen: {"kind":"ally","label":"Brannoch","distance":"86","edge":"left"},
  offscreenRight: {"kind":"danger","label":"Enemy","distance":"58","edge":"right"},
  overflow: {"label":"Light every lamp of the northern reach before nightfall","distance":"12,400","units":"km"},
};
