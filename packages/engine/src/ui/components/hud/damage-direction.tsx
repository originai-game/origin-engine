import { For, Line, View, t } from "@origingame/engine/ui";
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

export function DamageDirection(props: DamageDirectionProps) {
  return (
    <View class="relative w-full h-full min-w-[6.857rem] min-h-[6.857rem] pointer-events-none" role="image" label={t("shooter.damage.name")}>
      <For each={props.hits ?? []} key={(h) => h.id}>
        {(h) => {
          const strength = Math.min(Math.max(h().strength ?? 0, 0), 1);
          return (
            <View
              class="absolute inset-0 flex-row items-center justify-center transition duration-300 exit:opacity-0"
              style={{ opacity: 0.5 + strength * 0.5 }}
            >
              <View class="relative size-[6.25rem]">
                <Line
                  points={arcPoints(h().angle, strength)}
                  class={h().kind === "shield" ? "stroke-[#4fd8ff] stroke-[2.5]" : h().kind === "critical" ? "stroke-[#ff4d5e] stroke-[3.5]" : "stroke-[#ff4d5e] stroke-[2.5]"}
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
  empty: {"hits":[]},
  crowded: {"hits":[{"id":"a","angle":0,"strength":1,"kind":"critical"},{"id":"b","angle":1.57,"strength":0.7,"kind":"normal"},{"id":"c","angle":3.14,"strength":0.4,"kind":"normal"},{"id":"d","angle":-1.57,"strength":0.6,"kind":"shield"},{"id":"e","angle":-0.6,"strength":0.3,"kind":"normal"}]},
};
