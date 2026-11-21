import { Show, Text, View, formatDuration, packClass, t } from "@origingame/engine/ui";
import "./hud-timer.en.ogt";

export interface HudTimerProps {
  /** Seconds on the clock. */
  seconds: number;
  /** A countdown's whole time: a ring of what is left beside it (0: none). */
  total?: number;
  /** Counts down (warns and turns red near zero); false is a stopwatch. */
  countdown?: boolean;
  /** Seconds left when it turns warn; a third of it is critical. */
  warnAt?: number;
  /** Paused: the time dims and breathes. */
  paused?: boolean;
  /** A caption over it (Round 2); empty is none. */
  label?: string;
}

/**
 * A HUD clock on its plate: a caption, the seconds in the HUD's large
 * figure, and for a countdown with a total a ring of what is left, which
 * warns and then turns red near zero; paused, the time dims and breathes.
 * Read only.
 */
export function HudTimer(props: HudTimerProps) {
  const warn = () => (props.countdown ?? false) && props.seconds <= (props.warnAt ?? 0) && props.seconds > (props.warnAt ?? 0) / 3;
  const critical = () => (props.countdown ?? false) && props.seconds <= (props.warnAt ?? 0) / 3;
  return (
    <View
      class={`flex-col items-center gap-y-0.5 border-rim bg-linear-to-b from-plate-hi to-plate-lo shadow-plate py-2 px-4 data-paused:opacity-60 text-fg ${packClass("Plate", "frame")}`}
      role="timer"
      label={props.label || t("component.hud-timer.name")}
      valueText={formatDuration(props.seconds)}
      data-warn={warn()}
      data-critical={critical()}
      data-paused={props.paused}
    >
      <Show when={props.label}>
        <Text class="text-xs font-semibold tracking-label uppercase whitespace-nowrap opacity-80 text-shadow-2xs">{props.label}</Text>
      </Show>
      <View class="flex-row items-center gap-1">
        <Show when={(props.countdown ?? false) && (props.total ?? 0) > 0}>
          <View class="group/hudtimer relative size-6" data-warn={warn()} data-critical={critical()}>
            <View
              class="absolute inset-0 nine-ring bg-gold group-data-warn/hudtimer:bg-warn group-data-critical/hudtimer:bg-danger transition-layout duration-180 ease-out"
              wipe={Math.min(Math.max(props.seconds / (props.total ?? 1), 0), 1)}
            />
          </View>
        </Show>
        <Text
          class={`text-2xl tabular-nums whitespace-nowrap text-shadow-2xs data-warn:text-warn data-critical:text-danger data-paused:animate-pulse data-critical:animate-beat`}
          data-warn={warn()}
          data-critical={critical()}
          data-paused={props.paused}
        >
          {formatDuration(props.seconds)}
        </Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudTimerProps>> = {
  default: {"seconds":8.4,"total":90,"countdown":true,"warnAt":10,"paused":false,"label":"Round 2"},
};
