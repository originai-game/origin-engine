import { For, Show, View, packClass, t } from "@origingame/engine/ui";
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
  /** The units' bars, keyed rows of { id, entity, at, value, max, side, resource, resourceMax }: entity the unit by its bits (0: at, a world point x y z); value its health out of max; side whose it is, own (green), ally (blue), enemy (red) or neutral (yellow); resource out of resourceMax a thin line under the health (mana, energy), resourceMax 0 draws none. */
  bars?: ReadonlyArray<OverheadBar>;
  /** World units over each unit's point (its head). */
  height?: number;
}

/**
 * A health bar over every unit in the side's colour with a draining trail
 * ghost and an optional thin resource line grouped under one world anchor.
 * Read only.
 */
export function OverheadBars(props: OverheadBarsProps) {
  const height = () => props.height ?? 0;
  return (
    <View role="group" label={t("world.bars.name")}>
      <For each={props.bars ?? []} key={(b) => b.id}>
        {(b) => {
          const point = () => b().at.split(/\s+/).map(Number) as [number, number, number];
          const anchor = () => (b().entity ? (b().entity as Entity) : point());
          const share = () => (b().max > 0 ? Math.min(Math.max(b().value / b().max, 0), 1) : 0);
          const resourceShare = () =>
            (b().resourceMax ?? 0) > 0
              ? Math.min(Math.max((b().resource ?? 0) / (b().resourceMax ?? 1), 0), 1)
              : 0;
          return (
            <View
              anchor={anchor()}
              lift={[0, height(), 0]}
              occlusion="fade"
              declutter="stack"
              class="-translate-x-1/2 flex-col items-center gap-y-0.5"
            >
              <View class="relative h-1.25 w-11.5 overflow-hidden bg-track ring ring-rim">
                <View
                  class={`absolute left-0 top-0 h-full ${packClass("Bar", "ghost")} ${packClass("Bar", "trail")}`}
                  style={{ width: `${share() * 100}%` }}
                />
                <View
                  class={`absolute left-0 top-0 h-full bg-success data-ally:bg-info data-enemy:bg-danger data-neutral:bg-gold ${packClass("Bar", "fill")}`}
                  data-ally={b().side === "ally"}
                  data-enemy={b().side === "enemy"}
                  data-neutral={b().side === "neutral"}
                  style={{ width: `${share() * 100}%` }}
                />
              </View>
              <Show when={(b().resourceMax ?? 0) > 0}>
                <View class="relative h-0.75 w-9.25 overflow-hidden bg-track ring ring-rim">
                  <View
                    class={`absolute left-0 top-0 h-full bg-mana ${packClass("Bar", "fill")}`}
                    style={{ width: `${resourceShare() * 100}%` }}
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
