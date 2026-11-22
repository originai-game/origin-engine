import { Text, View, t, packClass } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingPedalsProps {
  /** The throttle's travel now, 0 to 1. */
  throttle: number;
  /** The brake's travel now, 0 to 1. */
  brake: number;
}

/**
 * The pedals at the bottom left: throttle and brake as two leaning upright
 * wells filled from the foot, green and red, each named under it. Read
 * only.
 */
export function RacingPedals(props: RacingPedalsProps) {
  return (
    <View class={`flex-row items-end gap-x-3 text-fg compact:gap-x-2`} role="group" label={t("racing.pedals.name")}>
      <View class="flex-col items-center gap-y-1.25">
        <View
          class={`relative w-3 h-14.5 overflow-hidden bg-well ring ring-rim compact:w-2.25 compact:h-10`}
          role="meter"
          label={t("racing.pedals.throttleName")}
          valueMin={0}
          valueMax={100}
          valueNow={props.throttle <= 0 ? 0 : props.throttle >= 1 ? 100 : props.throttle * 100}
        >
          <View class="absolute bottom-0 left-0 w-full bg-boon" style={{ height: `${(props.throttle <= 0 ? 0 : props.throttle >= 1 ? 1 : props.throttle) * 100}%` }} />
        </View>
        <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted compact:tracking-normal py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>{t("racing.pedals.throttle")}</Text>
      </View>
      <View class="flex-col items-center gap-y-1.25">
        <View
          class={`relative w-3 h-14.5 overflow-hidden bg-well ring ring-rim compact:w-2.25 compact:h-10`}
          role="meter"
          label={t("racing.pedals.brakeName")}
          valueMin={0}
          valueMax={100}
          valueNow={props.brake <= 0 ? 0 : props.brake >= 1 ? 100 : props.brake * 100}
        >
          <View class="absolute bottom-0 left-0 w-full bg-foe" style={{ height: `${(props.brake <= 0 ? 0 : props.brake >= 1 ? 1 : props.brake) * 100}%` }} />
        </View>
        <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted compact:tracking-normal py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>{t("racing.pedals.brake")}</Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingPedalsProps>> = {
  default: { "throttle": 0.86, "brake": 0.12 },
  braking: { "throttle": 0, "brake": 1 },
  empty: { "throttle": 0, "brake": 0 },
  overflow: { "throttle": 4, "brake": -2 },
};
