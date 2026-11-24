import { For, Text, formatNumber, packClass } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";

export interface DamageHit {
  id: string;
  /** The unit hit, by its bits (0: at, a world point x y z). */
  entity: number;
  /** A world point x y z without an entity. */
  at: string;
  /** The damage, shown as the number. */
  amount: number;
  /** The number's ink as #rrggbb (empty: its kind's). */
  color?: string;
  /** normal (empty), crit (larger and gold), heal, shield, weak or miss: the theme's look for each (Damage.number.*). */
  kind?: string;
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
const KINDS = ["crit", "heal", "shield", "weak", "miss"];

function parsePoint(at: string): [number, number, number] {
  const parts = (at || "").trim().split(/\s+/).map(Number);
  return [
    Number.isFinite(parts[0]) ? parts[0]! : 0,
    Number.isFinite(parts[1]) ? parts[1]! : 0,
    Number.isFinite(parts[2]) ? parts[2]! : 0,
  ];
}

function formatHit(hit: DamageHit): string {
  if (hit.kind === "miss") return "MISS";
  const num = formatNumber(hit.amount);
  if (hit.kind === "heal") return `+${num}`;
  if (hit.kind === "crit") return `${num}!`;
  return num;
}

export function DamageNumbers(props: DamageNumbersProps) {
  return (
    <For each={props.hits ?? []} key={(h) => h.id}>
      {(h) => (
        <Text
          anchor={h().entity ? (h().entity as Entity) : parsePoint(h().at)}
          declutter="stack"
          class={`-translate-x-1/2 pointer-events-none whitespace-nowrap ${packClass("Damage", "number")} ${KINDS.includes(h().kind ?? "") ? packClass("Damage", "number", h().kind) : ""}`}
          style={{ color: h().color || undefined }}
        >
          {formatHit(h())}
        </Text>
      )}
    </For>
  );
}

export const fixtures: Record<string, Partial<DamageNumbersProps>> = {
  default: {"hits":[{"id":"1","entity":0,"at":"0 2 0","amount":128,"color":""},{"id":"2","entity":0,"at":"0.5 2 0","amount":1240,"color":"","kind":"crit"},{"id":"3","entity":0,"at":"-0.6 1.6 0","amount":40,"color":"","kind":"heal"}]},
};
