import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./progress-ring.en.ogt";

export interface ProgressRingProps {
  /** Done, 0 to 1. */
  value: number;
  /** Working with no known value: the arc turns (indeterminate). */
  spinning?: boolean;
  /** Text in the middle; empty is the percent (none while spinning). */
  label?: string;
  /** What a screen reader calls it; empty is Progress, or Loading while spinning. */
  name?: string;
  /** The fill's tone: primary (empty), success, warn, danger or info. */
  tone?: string;
}

/**
 * A ring that fills with its value, its percent (or the label's words) in
 * the middle; with no known value the ring turns instead. Its side,
 * track, fill, spin and label are the Ring component's pack slots. Read
 * only.
 */
export function ProgressRing(props: ProgressRingProps) {
  const share = () => Math.min(Math.max(props.value, 0), 1);
  return (
    <View
      class={`group/ring relative ${packClass("Ring", "box")}`}
      role="progressbar"
      label={(props.name ?? "") !== "" ? props.name : props.spinning ? t("component.progress-ring.loading") : t("component.progress-ring.name")}
      valueMin={0}
      valueMax={100}
      valueNow={props.value * 100}
      busy={props.spinning}
      data-tone-success={(props.tone ?? "") === "success"}
      data-tone-warn={(props.tone ?? "") === "warn"}
      data-tone-danger={(props.tone ?? "") === "danger"}
      data-tone-info={(props.tone ?? "") === "info"}
    >
      <Show when={props.spinning} fallback={
        <>
          <View class={`absolute inset-0 ${packClass("Ring", "track")}`}>
            <View class={`absolute inset-0 ${packClass("Ring", "rest")}`} />
            <View
              class={`absolute inset-0 ${packClass("Ring", "fill")} ${["success", "warn", "danger", "info"].includes(props.tone ?? "") ? packClass("Ring", "fill", props.tone) : ""}`}
              wipe={share()}
            />
          </View>
          <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
            <Text class={`whitespace-nowrap ${packClass("Ring", "label")}`}>
              {(props.label ?? "") !== "" ? props.label : `${formatNumber(share() * 100, { decimals: 0 })}%`}
            </Text>
          </View>
        </>
      }>
        <View class={`absolute inset-0 ${packClass("Ring", "spin")} border-t-accent group-data-tone-success/ring:border-t-success group-data-tone-warn/ring:border-t-warn group-data-tone-danger/ring:border-t-danger group-data-tone-info/ring:border-t-info`}>
          <Show when={(props.label ?? "") !== ""}>
            <View class="absolute inset-0 flex-row items-center justify-center">
              <Text class={`whitespace-nowrap ${packClass("Ring", "label")}`}>{props.label}</Text>
            </View>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ProgressRingProps>> = {
  default: {"value":0.62,"spinning":false,"label":"","name":"","tone":""},
  spinning: {"value":0,"spinning":true,"label":"","name":"","tone":""},
  labelled: {"value":0.3,"spinning":false,"label":"Saving","name":"","tone":""},
  danger: {"value":0.8,"spinning":false,"label":"","name":"","tone":"danger"},
};
