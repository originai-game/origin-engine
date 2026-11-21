import { packClass, Show, t, Text, View } from "@origingame/engine/ui";
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
      class={`group/clock flex-row items-center gap-x-[0.714rem] w-fit max-w-full min-w-0 py-[0.429rem] px-4 text-fg nine-plate-cut bg-plate-hi compact:w-full compact:gap-x-[0.429rem] compact:py-[0.286rem] compact:px-[0.714rem] ${packClass("Plate", "frame")}`}
      role="timer"
      label={props.label !== "" ? props.label : t("strategy.clock.name")}
      valueText={props.time}
      data-paused={props.paused}
    >
      <Show when={props.label !== ""}>
        <Text class={`min-w-0 text-xs font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg/78 compact:text-xs compact:tracking-normal`}>
          {props.label}
        </Text>
      </Show>
      <Text class={`shrink text-lg leading-[1.1] font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg compact:text-reading`}>
        {props.time}
      </Text>
      <Show when={props.paused} fallback={
          <Show when={props.speed !== ""}>
            <Text class={`shrink min-w-[1.571rem] py-0 px-[0.429rem] leading-[1.5] text-xs font-bold tabular-nums text-center whitespace-nowrap nine-chip-cut bg-well ${packClass("Chip", "frame")}`}>
              {props.speed}
            </Text>
          </Show>
      }>
        <Text class={`shrink min-w-[1.571rem] py-0 px-[0.429rem] leading-[1.5] text-xs font-bold tabular-nums text-center whitespace-nowrap text-gold nine-chip-cut bg-well ${packClass("Chip", "frame")}`} role="status">
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
