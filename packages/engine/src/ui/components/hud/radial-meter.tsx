import { Show, View, t } from "@origingame/engine/ui";
import "./radial-meter.en.ogt";

export interface RadialMeterProps {
  /** The value in wheels: 1 is one full wheel, up to 3. */
  value?: number;
  /** Wheels (an extended stamina's second and third ring), 1 to 3. Default 1. */
  wheels?: number;
  /** Ran out: red until full again. */
  exhausted?: boolean;
  /** Faded out (the game hides it once it has been full a while). */
  hidden?: boolean;
  /** What a screen reader calls it; empty is Stamina. */
  name?: string;
}

/**
 * Stamina's wheels beside the scene: up to three concentric rings, each
 * emptying as its wheel is spent, red once the meter has run out and gone
 * when the game hides it. The style draws SVG ring arcs; here each wheel
 * is the pack's ring art, inset so its width matches the arc's, emptying
 * with its wipe. The wheels are nested largest first in one --size-lg box,
 * as the style's 56-space rings are.
 */
function clamp01(v: number): number {
  return Math.min(Math.max(v, 0), 1);
}

function RadialMeterWheel(props: { size: string; inset: string; fill: number; exhausted: boolean }) {
  const ink = () => (props.exhausted ? "bg-foe" : "bg-success");
  return (
    <View class={`absolute ${props.size}`}>
      <View class="absolute inset-0 rounded-full border border-rim shadow-[0px_0px_0px_1px_#00000073] bg-lit" />
      <View class={`absolute nine-ring ${ink()} transition-layout duration-180 ease-out`} style={{ inset: props.inset }} wipe={props.fill} />
      <View class="absolute inset-0 rounded-full bg-linear-to-b from-sheen to-transparent to-42%" />
    </View>
  );
}

export function RadialMeter(props: RadialMeterProps) {
  const value = () => props.value ?? 0;
  const wheels = () => Math.min(Math.max(props.wheels ?? 1, 1), 3);
  return (
    <View
      class="relative size-12 pointer-events-none data-gone:opacity-0"
      data-gone={props.hidden === true}
      data-exhausted={props.exhausted === true}
      role="meter"
      label={props.name || t("component.radial-meter.stamina")}
      valueMin={0}
      valueMax={wheels()}
      valueNow={value()}
    >
      <Show when={wheels() >= 1}>
        <RadialMeterWheel size="left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 absolute size-[2.071rem]" inset="13.4%" fill={clamp01(value())} exhausted={props.exhausted === true} />
      </Show>
      <Show when={wheels() >= 2}>
        <RadialMeterWheel size="left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 absolute size-[2.457rem]" inset="6.25%" fill={clamp01(value() - 1)} exhausted={props.exhausted === true} />
      </Show>
      <Show when={wheels() >= 3}>
        <RadialMeterWheel size="left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 absolute size-[2.929rem]" inset="0%" fill={clamp01(value() - 2)} exhausted={props.exhausted === true} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RadialMeterProps>> = {
  default: {"value":1.4,"wheels":2,"exhausted":false,"hidden":false,"name":""},
};
