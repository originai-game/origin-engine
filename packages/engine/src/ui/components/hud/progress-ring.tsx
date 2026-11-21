import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
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
 * the middle; with no known value the ring turns instead. The style's SVG
 * arc is the pack's ring art here, and the spinning ring turns as a lit
 * rim. Read only.
 */
export function ProgressRing(props: ProgressRingProps) {
  const share = () => Math.min(Math.max(props.value, 0), 1);
  return (
    <View
      class={`group/ring relative size-12`}
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
          <View class="absolute inset-0 rounded-full bg-glass">
            <View
              class={`absolute inset-0 nine-ring bg-accent group-data-tone-success/ring:bg-success group-data-tone-warn/ring:bg-warn group-data-tone-danger/ring:bg-danger group-data-tone-info/ring:bg-info transition-layout duration-300 ease-out`}
              wipe={share()}
            />
          </View>
          <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
            <Text class="text-reading tabular-nums whitespace-nowrap text-fg text-shadow-xs">
              {(props.label ?? "") !== "" ? props.label : `${formatNumber(share() * 100, { decimals: 0 })}%`}
            </Text>
          </View>
        </>
      }>
        <View class={`absolute inset-0 rounded-full border-4 border-rim border-t-accent group-data-tone-success/ring:border-t-success group-data-tone-warn/ring:border-t-warn group-data-tone-danger/ring:border-t-danger group-data-tone-info/ring:border-t-info animate-spin`}>
          <Show when={(props.label ?? "") !== ""}>
            <View class="absolute inset-0 flex-row items-center justify-center">
              <Text class="text-reading whitespace-nowrap text-fg text-shadow-xs">{props.label}</Text>
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
