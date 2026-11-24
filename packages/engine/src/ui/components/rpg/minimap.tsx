import type { Texture } from "@origingame/engine";
import { For, Image, Line, packClass, Show, t, Text, View } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgMinimapProps {
  /** The map texture (og.load.texture); 0 draws the dark well alone. */
  map: Texture | 0;
  /** The map point under the player, 0 to 1 across. Default 0.5. */
  centerX?: number;
  /** The map point under the player, 0 to 1 down. Default 0.5. */
  centerY?: number;
  /** How much of the map the minimap shows across, 0 to 1 (0.5). Default 0.5. */
  zoom?: number;
  /** The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. */
  heading: number;
  /** What the minimap shows, keyed rows of { id, kind, x, y, label }: kind ally, foe, boss or objective (on data-kind, each its own shape and colour); x and y on the map, 0 to 1; label names it for a reader. A blip off the minimap's edge is clipped. */
  blips?: ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string }>;
  /** Where the player is as the game words it (Ashen Vale), under the map; empty hides it. */
  place: string;
  /** The world's time of day as the game words it (06:40), on a tag beside the place; empty hides it. */
  clock: string;
}

/**
 * The minimap at the screen's top corner: the map in a dark round well
 * with a lit ring, the party, the foes and the objective as blips, the
 * player's arrow at the centre turned to the heading and N on the ring's
 * top; the place under it with the time of day on a tag. Read only.
 */
export function RpgMinimap(props: RpgMinimapProps) {
  const zoom = () => ((props.zoom ?? 0.5) > 0 ? (props.zoom ?? 0.5) : 1);
  return (
    <View class={`flex-col items-center gap-y-2 max-w-full shrink-0 text-fg compact:gap-y-1.5`} role="group" label={props.place !== "" ? props.place : t("rpg.map.name")}>
      <View class="relative size-38.5 compact:size-25.5">
        <View class="absolute inset-0 overflow-hidden rounded-full bg-well shadow-minimap" role="image" label={t("rpg.map.name")}>
          <Show when={props.map}>
            <Image class="absolute" style={{ width: `${100 / zoom()}%`, height: `${100 / zoom()}%`, left: `${50 - (props.centerX ?? 0.5) * 100 / zoom()}%`, top: `${50 - (props.centerY ?? 0.5) * 100 / zoom()}%` }} src={props.map} />
          </Show>
          <For each={props.blips ?? []} key={(b) => b.id}>
            {(b) => (
            <View class="absolute" style={{ left: `${50 + (b().x - (props.centerX ?? 0.5)) * 100 / zoom()}%`, top: `${50 + (b().y - (props.centerY ?? 0.5)) * 100 / zoom()}%` }}>
              <View class={`-translate-x-1/2 -translate-y-1/2 size-2 rounded-full ring-regular ring-shade-deep data-ally:bg-info data-foe:rounded-none data-foe:rotate-45 data-foe:bg-danger data-foe:shadow-glow-sm data-foe:shadow-foe data-foe:ring-regular data-foe:ring-shade-deep data-boss:size-2.75 data-boss:rounded-none data-boss:rotate-45 data-boss:bg-danger data-boss:shadow-glow-sm data-boss:shadow-foe data-boss:ring-regular data-boss:ring-shade-deep data-objective:size-2.75 data-objective:rounded-none data-objective:rotate-45 data-objective:bg-gold compact:size-2 compact:data-boss:size-2.5 compact:data-objective:size-2.5`}
                role="image" label={b().label}
                data-ally={b().kind === "ally"} data-foe={b().kind === "foe"} data-boss={b().kind === "boss"} data-objective={b().kind === "objective"} />
            </View>
            )}
          </For>
          {/* The player's arrow: the SVG wedge stands in as a stroked
              closed Line, turned to the heading. */}
          <View class="absolute left-1/2 top-1/2 size-3.25 compact:size-2.75" style={{ transform: `translate(-50%, -50%) rotate(${props.heading}deg)` }}>
            <Line class="size-full stroke-fg" points={[0.8, 12.2, 6.5, 0.8, 12.2, 12.2, 6.5, 8.9]} closed={true} />
          </View>
        </View>
        <View class="absolute top-0 inset-x-0 flex-row justify-center pointer-events-none">
          <Text class={`py-0 px-1 text-xs font-bold whitespace-nowrap ${packClass("Chip", "frame")}`}>
            {t("rpg.minimap.north")}
          </Text>
        </View>
      </View>
      <Show when={props.place !== "" || props.clock !== ""}>
        <View class="flex-row items-center justify-center max-w-full min-w-0 gap-1 compact:flex-wrap compact:gap-y-0">
          <Show when={props.place !== ""}>
            <Text class={`min-w-0 text-reading font-semibold whitespace-nowrap overflow-hidden text-fg text-stroke-3 text-stroke-bg compact:text-caption`}>
              {props.place}
            </Text>
          </Show>
          <Show when={props.clock !== ""}>
            <Text class={`shrink max-w-1/2 py-0 px-1 text-xs font-bold tabular-nums whitespace-nowrap overflow-hidden ${packClass("Chip", "frame")}`}>
              {props.clock}
            </Text>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgMinimapProps>> = {
  default: {"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":0,"place":"Ashen Vale","clock":"06:40","blips":[{"id":"boss","kind":"boss","x":0.56,"y":0.4,"label":"Ashen Colossus"},{"id":"gate","kind":"objective","x":0.44,"y":0.37,"label":"The Ninefold Gate"},{"id":"a1","kind":"ally","x":0.46,"y":0.55,"label":"Brannoch"},{"id":"a2","kind":"ally","x":0.53,"y":0.57,"label":"Sylvenne"}]},
  empty: {"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":0,"place":"","clock":"","blips":[]},
  edge: {"map":0,"centerX":0.05,"centerY":0.95,"zoom":0.9,"heading":135,"place":"Saltmarsh","clock":"","blips":[{"id":"far","kind":"foe","x":0.9,"y":0.1,"label":"Wolf"},{"id":"near","kind":"objective","x":0.1,"y":0.9,"label":"Shrine"}]},
  overflow: {"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":-30,"place":"The Drowned Abbey of Saint Wendreth beneath the Mere","clock":"Night, 23:59:59","blips":[]},
};
