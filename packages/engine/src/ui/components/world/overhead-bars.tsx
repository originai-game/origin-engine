import { For, Show, View, t } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface OverheadBar {
  id: string;
  /** The unit, by its bits (0: at, a world point x y z). */
  entity: number;
  /** A world point x y z without an entity. */
  at: string;
  /** Its health out of max. */
  value: number;
  max: number;
  /** Whose it is: own (green), ally (blue), enemy (red) or neutral (yellow). */
  side?: string;
  /** A thin resource line under the health (mana, energy). */
  resource?: number;
  resourceMax?: number;
}

export interface OverheadBarsProps {
  /** The units' bars, keyed rows of { id, entity, at, value, max, side, resource, resourceMax }: entity the unit by its bits (0: at, a world point x y z); value its health out of max; side whose it is, own (green), ally (blue), enemy (red) or neutral (yellow); resource out of resourceMax a thin line under the health (mana, energy), resourceMax 0 draws none. Thousands are fine: each bar is a GPU mass element of a fixed size (never under 30 × 3 px) that keeps the ghost of a hit. */
  bars?: ReadonlyArray<OverheadBar>;
  /** World units over each unit's point (its head). */
  height?: number;
}

/**
 * A health bar over every unit in the side's colour, a thin resource line
 * under it. Read only. The old anchors were GPU mass elements (one
 * instanced draw for a battle's thousands, a hit leaving a cream ghost):
 * each bar is now a text-free pair of nodes and the ghost is dropped.
 */
export function OverheadBars(props: OverheadBarsProps) {
  const height = () => props.height ?? 0;
  return (
    <View role="group" label={t("world.bars.name")}>
      <For each={props.bars ?? []} key={(b) => b.id}>
        {(b) => {
          const point = b().at.split(/\s+/).map(Number) as [number, number, number];
          const anchor = b().entity ? (b().entity as Entity) : point;
          const share = () => (b().max > 0 ? Math.min(Math.max(b().value / b().max, 0), 1) : 0);
          return (
            <View>
              <View
                anchor={anchor}
                lift={[0, height(), 0]}
                class="-translate-x-1/2 relative h-[0.357rem] w-[3.286rem] bg-tile shadow-[0px_0px_0px_1px_#ffffff47]"
              >
                <View
                  class="absolute left-0 top-0 h-full bg-[#55e39a] data-ally:bg-[#6ea8ff] data-enemy:bg-[#ff4d5e] data-neutral:bg-[#ffd23f]"
                  data-ally={b().side === "ally"}
                  data-enemy={b().side === "enemy"}
                  data-neutral={b().side === "neutral"}
                  style={{ width: `${share() * 100}%` }}
                />
              </View>
              <Show when={(b().resourceMax ?? 0) > 0}>
                <View
                  anchor={anchor}
                  lift={[0, height() - 0.12, 0]}
                  class="-translate-x-1/2 relative h-[0.214rem] w-[2.643rem] bg-tile shadow-[0px_0px_0px_1px_#ffffff47]"
                >
                  <View
                    class="absolute left-0 top-0 h-full bg-[#78b2ff]"
                    style={{ width: `${(b().resourceMax ?? 0) > 0 ? Math.min(Math.max((b().resource ?? 0) / (b().resourceMax ?? 1), 0), 1) * 100 : 0}%` }}
                  />
                </View>
              </Show>
            </View>
          );
        }}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<OverheadBarsProps>> = {
  default: {"height":2,"bars":[{"id":"a","entity":0,"at":"-1 0 0","value":90,"max":100,"side":"own","resource":70,"resourceMax":100},{"id":"b","entity":0,"at":"1 0 0","value":24,"max":100,"side":"ally","resource":35,"resourceMax":100}]},
  sides: {"height":2,"bars":[{"id":"o","entity":0,"at":"-3 0 0","value":100,"max":100,"side":"own","resource":0,"resourceMax":0},{"id":"a","entity":0,"at":"-1 0 0","value":70,"max":100,"side":"ally","resource":0,"resourceMax":0},{"id":"e","entity":0,"at":"1 0 0","value":45,"max":100,"side":"enemy","resource":0,"resourceMax":0},{"id":"n","entity":0,"at":"3 0 0","value":88,"max":100,"side":"neutral","resource":0,"resourceMax":0}]},
  low: {"height":2,"bars":[{"id":"l","entity":0,"at":"0 0 0","value":12,"max":100,"side":"enemy","resource":0,"resourceMax":0}]},
};
