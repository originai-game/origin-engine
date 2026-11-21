import { For, Text, formatNumber } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";

export interface DamageHit {
  id: string;
  /** The unit hit, by its bits (0: at, a world point x y z). */
  entity: number;
  /** A world point x y z without an entity. */
  at: string;
  /** The damage, shown as the number. */
  amount: number;
  /** The number's ink as #rrggbb (empty is white). */
  color?: string;
}

export interface DamageNumbersProps {
  /** The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades. */
  hits?: ReadonlyArray<DamageHit>;
}

/**
 * Damage numbers rising from where each hit landed: one number per hit,
 * anchored at the unit (or the world point) in the hit's colour, held in
 * the tag halo so it reads over the scene. Each new row plays the float
 * (rise and fade) once. Read only. The old anchor's pooled GPU mass
 * figures (one instanced draw for a battle's numbers, the rise eased on
 * the GPU) have no TSX surface: each number is a text node and the float
 * runs on the built-in animate-float.
 */
export function DamageNumbers(props: DamageNumbersProps) {
  return (
    <For each={props.hits ?? []} key={(h) => h.id}>
      {(h) => {
        const point = h().at.split(/\s+/).map(Number) as [number, number, number];
        return (
          <Text
            anchor={h().entity ? (h().entity as Entity) : point}
            class="-translate-x-1/2 animate-float text-lg font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs"
            style={{ color: h().color || undefined }}
          >
            {formatNumber(h().amount)}
          </Text>
        );
      }}
    </For>
  );
}

export const fixtures: Record<string, Partial<DamageNumbersProps>> = {
  default: {"hits":[{"id":"1","entity":0,"at":"0 2 0","amount":128,"color":""},{"id":"2","entity":0,"at":"0.5 2 0","amount":1240,"color":"#ffc93c"}]},
};
