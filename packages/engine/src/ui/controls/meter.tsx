import { View } from "@origingame/engine/ui";

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
  const fraction = () => {
    const min = props.min ?? 0;
    const max = props.max ?? 1;
    return max > min ? Math.min(Math.max((props.value - min) / (max - min), 0), 1) : 0;
  };
  return (
    <View
      role="meter"
      label={props.label}
      class={`group/meter w-40 h-3 overflow-hidden rounded-capsule bg-[#73798532] shadow-sunk ${props.class ?? ""}`}
      data-suboptimum={props.region === "suboptimum"}
      data-poor={props.region === "poor"}
    >
      <View
        class={`h-full rounded-capsule bg-success shadow-[inset_0px_1px_0px_0px_#ffffff0f] transition-layout duration-180 ease-out data-empty:hidden group-data-suboptimum/meter:bg-warn group-data-poor/meter:bg-danger`}
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
