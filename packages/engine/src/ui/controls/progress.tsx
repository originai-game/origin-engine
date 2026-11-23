import { View, packClass } from "@origingame/engine/ui";

export interface ProgressProps {
  /** How far the work is, as a percentage from 0 to 100 (clamped): NSH base's and gpui-base's range. */
  value: number;
  /** The extent is unknown (connecting, preparing): screen readers hear no value, and the indicator is the pack's `Progress.indeterminate`. */
  indeterminate?: boolean | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** What screen readers say is in progress. */
  label?: string | undefined;
}

/**
 * A linear progress bar (NSH base's `Progress`): it owns its role, its
 * range 0 to 100 and the indicator's width by the value; the pack draws its
 * track (`Progress.track`) and its indicator (`Bar.fill`, which eases as the
 * value moves).
 */
export function Progress(props: ProgressProps) {
  const percent = () => Math.min(Math.max(props.value, 0), 100);
  return (
    <View
      role="progressbar"
      label={props.label}
      valueMin={props.indeterminate ? undefined : 0}
      valueMax={props.indeterminate ? undefined : 100}
      valueNow={props.indeterminate ? undefined : percent()}
      busy={props.indeterminate}
      class={`${packClass("Progress", "track")} ${props.class ?? ""}`}
    >
      <View
        class={`h-full min-w-2 rounded-capsule bg-accent inset-shadow-lit ${packClass("Bar", "fill")} data-empty:hidden ${props.indeterminate ? packClass("Progress", "indeterminate") : ""}`}
        data-empty={!props.indeterminate && percent() <= 0}
        style={props.indeterminate ? undefined : { width: `${percent()}%` }}
      />
    </View>
  );
}

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<ProgressProps>> = {
  empty: { value: 0 },
  third: { value: 33 },
  full: { value: 100 },
  indeterminate: { value: 0, indeterminate: true, label: "Connecting" },
};
