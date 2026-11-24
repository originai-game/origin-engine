import { For, Line, View, packClass, t } from "@origingame/engine/ui";
import "./damage-direction.en.ogt";

export interface DamageDirectionHit {
  id: string;
  /** In radians from straight ahead, clockwise, relative to the view. */
  angle: number;
  /** 0 to 1: the arc's opacity and length. */
  strength?: number;
  /** normal, shield (the shield took it) or critical. */
  kind?: string;
}

export interface DamageDirectionProps {
  /** Recent hits. The game removes a hit when its mark should go. */
  hits?: ReadonlyArray<DamageDirectionHit>;
}

/**
 * A stretch of a ring around the view's centre for each recent hit,
 * centred where the damage came from, its length and weight by the hit's
 * strength, red with a glow (the shield's blue when the shield took it,
 * a critical's heavier). Read only. The style's SVG arcs and its glow
 * filter have no TSX surface: each arc is a sampled <Line> polyline in a
 * fixed 6.25rem ring (the .svelte scales its ring with its place), and
 * the glow is dropped.
 */
const TAU = Math.PI * 2;

function arcPoints(angle: number, strength: number): readonly number[] {
  const half = (0.045 + 0.035 * strength) * TAU;
  const points: number[] = [];
  const steps = 16;
  for (let k = 0; k <= steps; k++) {
    const theta = angle - half + (k * 2 * half) / steps;
    points.push(50 + 46 * Math.sin(theta), 50 - 46 * Math.cos(theta));
  }
  return points;
}

/**
 * Where recent hits came from: a mark at the screen's edge toward each hit.
 * The game adds a hit and removes it when its mark should go. Read only.
 */
export function DamageDirection(props: DamageDirectionProps) {
  return (
    <View class="relative w-full h-full min-w-24 min-h-24 pointer-events-none" role="image" label={t("shooter.damage.name")}>
      <For each={props.hits ?? []} key={(h) => h.id}>
        {(h) => {
          const strength = () => Math.min(Math.max(h().strength ?? 0, 0), 1);
          return (
            <View
              class="absolute inset-0 flex-row items-center justify-center"
              style={{ opacity: 0.5 + strength() * 0.5 }}
            >
              <View class={`relative ${packClass("Hit", "ring")}`}>
                <Line
                  viewBox="0 0 100 100"
                  points={arcPoints(h().angle, strength())}
                  class={`w-full h-full ${packClass("Hit", "arc")} ${h().kind === "shield" || h().kind === "critical" ? packClass("Hit", "arc", h().kind) : ""}`}
                />
              </View>
            </View>
          );
        }}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<DamageDirectionProps>> = {
  default: {"hits":[{"id":"h1","angle":0.9,"strength":0.9,"kind":"normal"},{"id":"h2","angle":-2.4,"strength":0.5,"kind":"shield"}]},
  crowded: {"hits":[{"id":"a","angle":0,"strength":1,"kind":"critical"},{"id":"b","angle":1.57,"strength":0.7,"kind":"normal"},{"id":"c","angle":3.14,"strength":0.4,"kind":"normal"},{"id":"d","angle":-1.57,"strength":0.6,"kind":"shield"},{"id":"e","angle":-0.6,"strength":0.3,"kind":"normal"}]},
};
