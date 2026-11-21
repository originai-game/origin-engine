import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmSongProgressProps {
  /** How far through the song, 0 to 1. */
  value: number;
  /** The time played as the game words it (2:07); empty hides the time. */
  elapsed: string;
  /** The song's length as the game words it (3:21), after the time played. */
  length: string;
}

/**
 * The song's progress as a thin lit line along the screen's top edge, and
 * the time played over the length riding under its played end. The game
 * keeps time; read only.
 */
export function RhythmSongProgress(props: RhythmSongProgressProps) {
  const full = props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value;
  return (
    <View
      class={`flex-col w-full`}
      role="progressbar"
      label={t("rhythm.hud.progress")}
      valueMin={0}
      valueMax={100}
      valueNow={props.value <= 0 ? 0 : props.value >= 1 ? 100 : props.value * 100}
      valueText={props.elapsed !== "" ? t("rhythm.progress.time", { elapsed: props.elapsed, length: props.length }) : `${formatNumber(full * 100, { decimals: 0 })}%`}
    >
      <View class={`relative w-full h-[0.357rem] bg-[#04060cb2]`}>
        <View class={`absolute left-0 top-0 h-full bg-linear-to-r from-[#38e8ff] to-white shadow-[0px_0px_12px_#38e8ff]`} style={{ width: `${full * 100}%` }} />
      </View>
      <Show when={props.elapsed !== ""}>
        <Show when={full < 0.9} fallback={
          <View class={`flex-row w-full px-3 mt-1 justify-end`}>
            <View class={`flex-row items-baseline gap-0.5 shrink`}>
              <Text class={`text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55`}>{props.elapsed}</Text>
              <Show when={props.length !== ""}>
                <Text class={`text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg-muted text-shadow-xs text-shadow-bg/55`}>
                  {t("rhythm.progress.length", { length: props.length })}
                </Text>
              </Show>
            </View>
          </View>
        }>
          <View class={`flex-row w-full px-3 mt-1`}>
            <View class={`flex-row items-baseline gap-0.5 shrink`} style={{ marginLeft: `${full * 100}%` }}>
              <Text class={`text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55`}>{props.elapsed}</Text>
              <Show when={props.length !== ""}>
                <Text class={`text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg-muted text-shadow-xs text-shadow-bg/55`}>
                  {t("rhythm.progress.length", { length: props.length })}
                </Text>
              </Show>
            </View>
          </View>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmSongProgressProps>> = {
  default: {"value":0.63,"elapsed":"2:07","length":"3:21"},
  start: {"value":0,"elapsed":"0:00","length":"3:21"},
  end: {"value":1,"elapsed":"3:21","length":"3:21"},
  bare: {"value":0.4,"elapsed":"","length":""},
  overflow: {"value":0.5,"elapsed":"1:02:44","length":"2:05:28"},
};
