import { Show, Text, View, t } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyClockProps {
  /** What the clock counts as the game words it (Day 3, Match), before the time; empty hides it. */
  label: string;
  /** The time as the game words it (12:34). */
  time: string;
  /** The game's speed as it words it (×1, ×2), on a tag after the time; empty hides it. */
  speed: string;
  /** The game is paused: the Paused word stands on the tag in place of the speed (data-paused). */
  paused: boolean;
}

/**
 * The match clock at the top right: what it counts (Day 3), the time large
 * and the game's speed on a tag, or Paused. The game owns the time and the
 * speed; read only.
 */
export function StrategyClock(props: StrategyClockProps) {
  return (
    <View
      class={`group/clock flex-row items-center gap-x-[0.714rem] w-fit max-w-full min-w-0 py-[0.429rem] px-[1rem] text-fg border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate border rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none compact:w-full compact:gap-x-[0.429rem] compact:py-[0.286rem] compact:px-[0.714rem]`}
      role="timer"
      label={props.label !== "" ? props.label : t("strategy.clock.name")}
      valueText={props.time}
      data-paused={props.paused}
    >
      <Show when={props.label !== ""}>
        <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-[#ffffffc7] compact:text-[0.857rem] compact:tracking-normal`}>
          {props.label}
        </Text>
      </Show>
      <Text class={`shrink text-[1.286rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg compact:text-[1.071rem]`}>
        {props.time}
      </Text>
      <Show when={props.paused} fallback={
          <Show when={props.speed !== ""}>
            <Text class={`shrink min-w-[1.571rem] py-0 px-[0.429rem] leading-[1.5] text-[0.857rem] font-bold tabular-nums text-center whitespace-nowrap bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`}>
              {props.speed}
            </Text>
          </Show>
      }>
        <Text class={`shrink min-w-[1.571rem] py-0 px-[0.429rem] leading-[1.5] text-[0.857rem] font-bold tabular-nums text-center whitespace-nowrap text-gold bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} role="status">
          {t("strategy.clock.paused")}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyClockProps>> = {
  default: {"label":"Day 3","time":"12:34","speed":"×1","paused":false},
  paused: {"label":"Day 3","time":"12:34","speed":"×1","paused":true},
  empty: {"label":"","time":"0:00","speed":"","paused":false},
  overflow: {"label":"Third Day of the Long Winter Campaign","time":"1,284:59:59","speed":"×16","paused":false},
};
