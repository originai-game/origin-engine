import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./progress-bar.en.ogt";

export interface ProgressBarProps {
  /** Done, 0 to 1. */
  value: number;
  /** A paler fill ahead of value (downloaded ahead of installed), 0 to 1. */
  buffer?: number;
  /** A caption over it at the start; empty is none. */
  label?: string;
  /** The value in words at the end; empty is the percent. */
  valueText?: string;
  /** Show the value at the end. */
  showValue?: boolean;
  /** Working with no known value: a band sweeps along the track. */
  indeterminate?: boolean;
  /** The fill's tone: primary (empty), success, warn, danger or info. */
  tone?: string;
}

/**
 * A bar fills the width its container gives it: a caption over it at the
 * start and its value in words at the end, a paler buffer ahead of the
 * fill in a dark well with one lit rim, thinner than a vital's; with no
 * known value a band sweeps the track instead. Drawn over the scene, its
 * words take the scene's opaque ink. The style's lean (skewX) and the
 * fill's sheen have no TSX surface: the lean is dropped and the sweeping
 * band carries the shimmer.
 */
export function ProgressBar(props: ProgressBarProps) {
  const percent = () => `${formatNumber(Math.min(Math.max(props.value, 0), 1) * 100, { decimals: 0 })}%`;
  return (
    <View
      class={`group/progress flex-col w-full min-w-0 gap-y-[0.286rem]`}
      role="progressbar"
      label={(props.label ?? "") !== "" ? props.label : t("component.progress-bar.name")}
      valueMin={0}
      valueMax={100}
      valueNow={props.value * 100}
      valueText={(props.valueText ?? "") !== "" ? props.valueText : percent()}
      busy={props.indeterminate}
      data-indeterminate={props.indeterminate}
      data-tone-success={(props.tone ?? "") === "success"}
      data-tone-warn={(props.tone ?? "") === "warn"}
      data-tone-danger={(props.tone ?? "") === "danger"}
      data-tone-info={(props.tone ?? "") === "info"}
    >
      <Show when={(props.label ?? "") !== "" || props.showValue}>
        <View class="flex-row items-end">
          <Show when={(props.label ?? "") !== ""}>
            <Text class="flex-1 min-w-0 text-[0.929rem] whitespace-nowrap truncate text-fg text-shadow-xs">{props.label}</Text>
          </Show>
          <Show when={props.showValue && !props.indeterminate}>
            <Text class="shrink-0 text-[14.88px] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{(props.valueText ?? "") !== "" ? props.valueText : percent()}</Text>
          </Show>
        </View>
      </Show>
      <View class="relative w-full h-[0.571rem] overflow-hidden bg-track shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24,0px_0.143rem_0.286rem_#00000073]">
        <Show when={!props.indeterminate}>
          <View
            class={`absolute left-0 top-0 h-full opacity-25 bg-accent group-data-tone-success/progress:bg-success group-data-tone-warn/progress:bg-warn group-data-tone-danger/progress:bg-danger group-data-tone-info/progress:bg-info`}
            style={{ width: `${Math.min(Math.max(props.buffer ?? 0, 0), 1) * 100}%` }}
          />
        </Show>
        <View
          class={`absolute left-0 top-0 h-full bg-linear-to-b from-sheen to-transparent to-42% bg-accent group-data-tone-success/progress:bg-none group-data-tone-success/progress:bg-success group-data-tone-warn/progress:bg-none group-data-tone-warn/progress:bg-warn group-data-tone-danger/progress:bg-none group-data-tone-danger/progress:bg-danger group-data-tone-info/progress:bg-none group-data-tone-info/progress:bg-info group-data-indeterminate/progress:animate-shimmer`}
          style={{ width: `${(props.indeterminate ? 0.36 : Math.min(Math.max(props.value, 0), 1)) * 100}%` }}
        />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ProgressBarProps>> = {
  default: {"value":0.42,"buffer":0.6,"label":"Downloading","valueText":"","showValue":true,"indeterminate":false,"tone":""},
  indeterminate: {"value":0,"buffer":0,"label":"Working","valueText":"","showValue":false,"indeterminate":true,"tone":""},
  success: {"value":1,"buffer":1,"label":"Installed","valueText":"","showValue":true,"indeterminate":false,"tone":"success"},
  danger: {"value":0.3,"buffer":0.3,"label":"Failed","valueText":"","showValue":true,"indeterminate":false,"tone":"danger"},
};
