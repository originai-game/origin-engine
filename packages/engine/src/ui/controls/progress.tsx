import { View, packClass } from "@origingame/engine/ui";

export interface ProgressProps {
  /** Done, 0 to 1. */
  value: number;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  label?: string | undefined;
}

/** A track filled to its value: a capsule well and its fill, which eases as the value moves. */
export function Progress(props: ProgressProps) {
  const fraction = () => Math.min(Math.max(props.value, 0), 1);
  return (
    <View role="progressbar" label={props.label} class={`w-full h-2 overflow-hidden rounded-capsule bg-fill-strong shadow-sunk ${props.class ?? ""}`}>
      <View
        class={`h-full min-w-2 rounded-capsule bg-accent inset-shadow-lit ${packClass("Bar", "fill")} data-empty:hidden`}
        data-empty={fraction() <= 0}
        style={{ width: `${fraction() * 100}%` }}
      />
    </View>
  );
}

export const fixtures: Record<string, Partial<ProgressProps>> = {
  empty: { value: 0 },
  third: { value: 0.33 },
  full: { value: 1 },
};
