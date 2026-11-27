import type { Texture } from "@origingame/engine";
import { For, Image, Show, Text, View, packClass } from "@origingame/engine/ui";
import "./world.en.strings.ts";

export interface MinimapMarker {
  id: string;
  /** Across the shown region, 0 to 1. */
  x: number;
  /** Down the shown region, 0 to 1. */
  y: number;
  /** neutral, info, success, warn, danger or accent. */
  tone?: string;
  /** The player's: the primary, larger and ringed. */
  player?: boolean;
}

export interface MinimapProps {
  /** The map texture (og.load.texture); 0 draws a grid over the stage colour. */
  texture?: Texture | 0;
  /** The region of the map shown: its left edge, 0 to 1. */
  u0?: number;
  /** Its top edge, 0 to 1 (v down). */
  v0?: number;
  /** Its right edge (1: the whole width). Default 1. */
  u1?: number;
  /** Its bottom edge (1: the whole height). Default 1. */
  v1?: number;
  /** Its side in px (160). */
  size?: number;
  /** Markers over the map. */
  markers?: ReadonlyArray<MinimapMarker>;
  /** A zone name under it; empty is none. */
  label?: string;
}

/**
 * The map of the place, its markers on it and a zone name under: the
 * texture or a grid over the stage colour when there is none. The region
 * crops by layout: the texture draws oversized and offset inside the
 * overflowing well. The grid's SVG is drawn as thin views on the tile.
 */
// Each tone's ground, a theme role paired per scheme by the pack.
const TONES: Record<string, string> = {
  info: "bg-info",
  success: "bg-success",
  warn: "bg-warn",
  danger: "bg-foe",
  accent: "bg-hud-accent",
};

function markerTone(m: MinimapMarker): string {
  return m.player ? "bg-hud-accent" : TONES[m.tone ?? ""] ?? "bg-fg";
}

/**
 * A square minimap: a region of the map texture (or a grid without one),
 * markers over it and a zone name under it on a plate. Read only.
 */
export function Minimap(props: MinimapProps) {
  const size = () => props.size ?? 160;
  return (
    <View class="flex-col items-center gap-y-3" role="image" label={props.label ?? ""}>
      <View class="relative overflow-hidden rounded-card inset-ring-hairline inset-ring-border shadow-plate" style={{ width: `${size()}px`, height: `${size()}px` }}>
        <Show when={(props.texture ?? 0) > 0} fallback={
          <View class="absolute inset-0 bg-tile">
            <For each={Array.from({ length: 7 }, (_, k) => k)} key={(k) => `h${k}`}>
              {(k) => <View class="absolute bg-rim" style={{ left: `${((k() + 1) / 8) * 100}%`, top: 0, width: `${100 / size()}%`, height: "100%" }} />}
            </For>
            <For each={Array.from({ length: 7 }, (_, k) => k)} key={(k) => `v${k}`}>
              {(k) => <View class="absolute bg-rim" style={{ top: `${((k() + 1) / 8) * 100}%`, left: 0, height: `${100 / size()}%`, width: "100%" }} />}
            </For>
          </View>
        }>
          <Image
            class="absolute"
            src={props.texture ?? 0}
            style={{
              width: `${100 / (Math.max((props.u1 ?? 1) - (props.u0 ?? 0), 0.0001))}%`,
              height: `${100 / (Math.max((props.v1 ?? 1) - (props.v0 ?? 0), 0.0001))}%`,
              left: `${(-(props.u0 ?? 0) / Math.max((props.u1 ?? 1) - (props.u0 ?? 0), 0.0001)) * 100}%`,
              top: `${(-(props.v0 ?? 0) / Math.max((props.v1 ?? 1) - (props.v0 ?? 0), 0.0001)) * 100}%`,
            }}
          />
        </Show>
        <For each={props.markers ?? []} key={(m) => m.id}>
          {(m) => (
            <View
              class={`absolute rounded-full ${markerTone(m())} ${m().player ? "size-3 border-regular border-accent" : "size-2"}`}
              style={{
                left: `${m().x * 100}%`,
                top: `${m().y * 100}%`,
                transform: "translate(-50%, -50%)",
              }}
            />
          )}
        </For>
      </View>
      <Show when={(props.label ?? "") !== ""}>
        <Text class={`py-0.5 px-2 whitespace-nowrap text-caption ${packClass("Plate", "frame")}`}>{props.label}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<MinimapProps>> = {
  default: {"texture":0,"u0":0,"v0":0,"u1":1,"v1":1,"size":160,"label":"Ashen Vale","markers":[{"id":"me","x":0.5,"y":0.5,"tone":"neutral","player":true},{"id":"a","x":0.25,"y":0.3,"tone":"danger","player":false},{"id":"b","x":0.72,"y":0.65,"tone":"success","player":false},{"id":"c","x":0.6,"y":0.2,"tone":"warn","player":false}]},
};
