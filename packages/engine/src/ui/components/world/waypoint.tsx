import { Icon, Show, Text, View, packClass } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.strings.ts";

export interface WaypointProps {
  /** What it marks, by its bits (0: use at). */
  entity?: number;
  /** A world point it marks without an entity (x y z). */
  at?: string;
  /** World units above the point. */
  height?: number;
  /** What it is: objective (empty, gold), ping (cyan, a ripple), danger (red), ally (green) or loot (violet). */
  kind?: string;
  /** Optional atlas icon rendered inside the marker. */
  icon?: string;
  /** Optional vertical elevation indicator relative to the player. */
  elevation?: "above" | "below" | "level";
  /** Its name on a HUD plate under the diamond (The Ninefold Gate); empty is none. It drops when far. */
  label?: string;
  /** How far it is as the game words it (412), on a dark tag; empty is none. */
  distance?: string;
  /** The distance's unit, smaller after it (m). */
  units?: string;
  /** near (a soft halo), mid (empty) or far (the label drops, the distance stays), as the game judges it. */
  range?: string;
  /** The side of the screen it is off, as the game reads it (left or right): a chevron toward it before the pin; empty is on screen. The anchor holds it inside the screen's edge. */
  edge?: string;
  /** Distance at which the marker is drawn at scale 1. */
  scaleAt?: number;
  /** Minimum world-distance scale. */
  scaleMin?: number;
  /** Maximum world-distance scale. */
  scaleMax?: number;
  /** Maximum world distance before hiding. */
  maxDistance?: number;
  /** Behaviour when behind world geometry. */
  occlusion?: "fade" | "hide" | "none";
}

function parsePoint(at?: string): [number, number, number] {
  const parts = (at ?? "0 0 0").trim().split(/\s+/).map(Number);
  return [
    Number.isFinite(parts[0]) ? parts[0]! : 0,
    Number.isFinite(parts[1]) ? parts[1]! : 0,
    Number.isFinite(parts[2]) ? parts[2]! : 0,
  ];
}

/**
 * A place to go: a lit diamond with a dark core (or custom icon) in the
 * kind's colour, optional elevation chevron, its name on a plate under it
 * and the distance on a dark tag. Off screen, a chevron points the way
 * and clamp="edge" holds it inside the screen's edge.
 */
export function Waypoint(props: WaypointProps) {
  const kind = () => ((props.kind ?? "") == "" ? "objective" : (props.kind ?? ""));
  const range = () => ((props.range ?? "") == "" ? "mid" : (props.range ?? ""));
  const edge = () => ((props.edge ?? "") == "" ? "none" : (props.edge ?? ""));
  const label = () => props.label ?? "";
  const distance = () => props.distance ?? "";
  const units = () => props.units ?? "";
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : parsePoint(props.at)}
      lift={[0, props.height ?? 0, 0]}
      clamp="edge"
      scaleAt={props.scaleAt}
      scaleMin={props.scaleMin}
      scaleMax={props.scaleMax}
      maxDistance={props.maxDistance}
      occlusion={props.occlusion}
      class="group -translate-x-1/2 -translate-y-1/2 pointer-events-none flex-col items-center gap-y-0.75"
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
      <Show when={props.elevation === "above"}>
        <View class="size-1.5 rotate-45 border-l-regular border-t-regular border-fg" />
      </Show>
      <View class="relative flex-row items-center justify-center gap-x-1">
        <View class="hidden group-data-way-left:block group-data-edge-left:block size-2 rotate-45 border-l-regular border-b-regular border-gold group-data-danger:border-danger group-data-ally:border-success group-data-loot:border-epic group-data-ping:border-rare" />
        <View class="relative flex-col items-center justify-center gap-y-1">
          <View class="hidden group-data-edge-top:block size-2 rotate-45 border-l-regular border-t-regular border-gold group-data-danger:border-danger group-data-ally:border-success group-data-loot:border-epic group-data-ping:border-rare" />
          <View class="relative m-1 size-5 items-center justify-center">
            <Show when={kind() === "ping"}>
              <View class="absolute inset-0 rotate-45 bg-rare opacity-soft animate-ping pointer-events-none" />
            </Show>
            <View class="absolute inset-0 rotate-45 bg-gold shadow-glow shadow-gold ring-regular ring-well group-data-danger:bg-danger group-data-danger:shadow-glow group-data-danger:shadow-foe group-data-danger:ring-regular group-data-danger:ring-well group-data-ally:bg-success group-data-ally:shadow-glow group-data-ally:shadow-boon group-data-ally:ring-regular group-data-ally:ring-well group-data-loot:bg-epic group-data-loot:shadow-glow group-data-loot:shadow-xp group-data-loot:ring-regular group-data-loot:ring-well group-data-ping:bg-rare group-data-ping:shadow-glow group-data-ping:shadow-info group-data-ping:ring-regular group-data-ping:ring-well group-data-near:shadow-glow-lg group-data-near:shadow-gold group-data-near:ring-regular group-data-near:ring-well">
              <Show when={!props.icon}>
                <View class="absolute inset-8/25 bg-tile" />
              </Show>
            </View>
            <Show when={props.icon}>
              <Icon class="relative size-3 text-bg" name={props.icon} />
            </Show>
          </View>
          <View class="hidden group-data-edge-bottom:block size-2 rotate-45 border-r-regular border-b-regular border-gold group-data-danger:border-danger group-data-ally:border-success group-data-loot:border-epic group-data-ping:border-rare" />
        </View>
        <View class="hidden group-data-way-right:block group-data-edge-right:block size-2 rotate-45 border-r-regular border-t-regular border-gold group-data-danger:border-danger group-data-ally:border-success group-data-loot:border-epic group-data-ping:border-rare" />
      </View>
      <Show when={props.elevation === "below"}>
        <View class="size-1.5 rotate-45 border-r-regular border-b-regular border-fg" />
      </Show>
      <Show when={(label() !== "" && range() !== "far") || (distance() !== "")}>
        <View class="flex-col items-center gap-y-0.5 min-w-0 group-data-way-left:items-start group-data-edge-left:items-start group-data-way-right:items-end group-data-edge-right:items-end">
          <Show when={label() !== "" && range() !== "far"}>
            <Text class={`min-w-0 max-w-50 py-0.25 px-1.5 text-2xs leading-tight font-bold tracking-wide uppercase whitespace-nowrap truncate ${packClass("Plate", "frame")}`}>{label()}</Text>
          </Show>
          <Show when={distance() !== ""}>
            <View class="shrink-0 flex-row items-baseline py-0.25 px-1.5 rounded-control bg-raised-pressed">
              <Text class="text-caption leading-tight font-bold tabular-nums whitespace-nowrap text-fg">{distance()}</Text>
              <Show when={units() !== ""}>
                <Text class="ml-px text-2xs leading-tight text-fg-muted">{units()}</Text>
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
