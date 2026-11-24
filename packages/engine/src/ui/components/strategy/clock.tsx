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
      class={`group/clock flex-row items-center gap-x-2.5 w-fit max-w-full min-w-0 py-1.5 px-4 compact:w-full compact:gap-x-1.5 compact:py-1 compact:px-2.5 ${packClass("Plate", "frame")}`}
      role="timer"
      label={props.label !== "" ? props.label : t("strategy.clock.name")}
      valueText={props.time}
      data-paused={props.paused}
    >
      <Show when={props.label !== ""}>
        <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden opacity-quiet compact:text-xs compact:tracking-normal`}>
          {props.label}
        </Text>
      </Show>
      <Text class={`shrink text-lg leading-tight font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg compact:text-reading`}>
        {props.time}
      </Text>
      <Show when={props.paused} fallback={
          <Show when={props.speed !== ""}>
            <Text class={`shrink min-w-5.5 py-0 px-1.5 leading-normal text-xs font-bold tabular-nums text-center whitespace-nowrap ${packClass("Chip", "frame")}`}>
              {props.speed}
            </Text>
          </Show>
      }>
        <Text class={`shrink min-w-5.5 py-0 px-1.5 leading-normal text-xs font-bold tabular-nums text-center whitespace-nowrap text-gold ${packClass("Chip", "frame")}`} role="status">
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
