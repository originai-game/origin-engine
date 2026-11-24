import { View, packClass } from "@origingame/engine/ui";

export interface MeterProps {
  value: number;
  /** The range; default 0 to 1. */
  min?: number | undefined;
  max?: number | undefined;
  /** Where the value lies against the meter's optimum, as the game judges it: the fill wears its colour. */
  region?: "optimum" | "suboptimum" | "poor" | undefined;
  class?: string | undefined;
  label?: string | undefined;
}

/** A track whose fill wears its value's region: good, middling or poor. */
export function Meter(props: MeterProps) {
  const min = () => (Number.isFinite(props.min) ? props.min! : 0);
  const max = () => (Number.isFinite(props.max) ? props.max! : 1);
  const value = () => (Number.isFinite(props.value) ? props.value : min());
  const fraction = () => (max() > min() ? Math.min(Math.max((value() - min()) / (max() - min()), 0), 1) : 0);
  return (
    <View
      role="meter"
      label={props.label}
      valueMin={min()}
      valueMax={max()}
      valueNow={value()}
      class={`group/meter w-40 h-3 overflow-hidden rounded-capsule bg-fill-strong shadow-sunk ${props.class ?? ""}`}
      data-suboptimum={props.region === "suboptimum"}
      data-poor={props.region === "poor"}
    >
      <View
        class={`h-full rounded-capsule bg-success inset-shadow-lit ${packClass("Bar", "fill")} data-empty:hidden group-data-suboptimum/meter:bg-warn group-data-poor/meter:bg-danger`}
        data-empty={fraction() <= 0}
        style={{ width: `${fraction() * 100}%` }}
      />
    </View>
  );
}

export const fixtures: Record<string, Partial<MeterProps>> = {
  optimum: { value: 0.8 },
  suboptimum: { value: 0.5, region: "suboptimum" },
  poor: { value: 0.2, region: "poor" },
};
