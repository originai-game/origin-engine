import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmScoreHudProps {
  /** The score as the game counts it, shown with grouping. */
  score: number;
  /** The accuracy as the game words it (98.42%); empty hides it. */
  accuracy: string;
  /** The grade the run is on course for as the game names it (S, AA); empty hides it. */
  grade: string;
  /** The player's best on this chart; 0 hides it. Ahead or behind it shows as a signed gap beside it (on data-trend). Default 0. */
  best?: number;
}

/**
 * The run's figures over the play field, set against the screen's end:
 * the score as the hero figure, the accuracy with the grade it is on course
 * for, and the player's best with the gap to it. The game owns every
 * figure; read only. Pair it with RhythmGauge for life and fever and
 * RhythmSongProgress for the song's progress.
 */
export function RhythmScoreHud(props: RhythmScoreHudProps) {
  return (
    <View class={`flex-col items-end gap-0 max-w-full`} role="group" label={t("rhythm.hud.name")}>
      <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg/78 text-stroke-3 text-stroke-bg`}>
        {t("rhythm.hud.score")}
      </Text>
      <Text class={`text-[4.714rem] compact:text-[2.714rem] leading-[0.9] font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg`}>
        {formatNumber(props.score)}
      </Text>
      <Show when={props.accuracy !== "" || props.grade !== ""}>
        <View class={`flex-row items-baseline justify-end gap-1 mt-1 compact:mt-0.5`}>
          <Show when={props.accuracy !== ""}>
            <Text
              class={`text-[1.714rem] compact:text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg`}
              label={t("rhythm.hud.accuracyName", { accuracy: props.accuracy })}
            >
              {props.accuracy}
            </Text>
          </Show>
          <Show when={props.grade !== ""}>
            <Text
              class={`text-[2.429rem] compact:text-[1.714rem] leading-none font-bold italic whitespace-nowrap text-gold text-stroke-3 text-stroke-bg`}
              label={t("rhythm.hud.gradeName", { grade: props.grade })}
            >
              {props.grade}
            </Text>
          </Show>
        </View>
      </Show>
      <Show when={(props.best ?? 0) > 0}>
        <View class={`flex-row items-baseline justify-end gap-1 mt-0.5`}>
          <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg/78 text-stroke-3 text-stroke-bg`}>
            {t("rhythm.hud.best")}
          </Text>
          <Text class={`text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg-muted text-stroke-3 text-stroke-bg`}>
            {formatNumber(props.best ?? 0)}
          </Text>
          <Text
            class={`shrink py-0 px-1 text-[0.857rem] font-bold tabular-nums whitespace-nowrap bg-[#060910e0] nine-chip-cut text-success data-trend-down:text-danger ${packClass("Chip", "frame")}`}
            data-trend-up={props.score >= (props.best ?? 0)}
            data-trend-down={props.score < (props.best ?? 0)}
          >
            {props.score >= (props.best ?? 0) ? t("rhythm.hud.ahead", { gap: props.score - (props.best ?? 0) }) : t("rhythm.hud.behind", { gap: (props.best ?? 0) - props.score })}
          </Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmScoreHudProps>> = {
  default: {"score":987450,"accuracy":"98.42%","grade":"S","best":991208},
  ahead: {"score":482310,"accuracy":"97.42%","grade":"AA","best":455000},
  first: {"score":215880,"accuracy":"81.05%","grade":"B","best":0},
  start: {"score":0,"accuracy":"","grade":"","best":0},
  overflow: {"score":123456789,"accuracy":"100.0000%","grade":"SSS+","best":123450000},
};
