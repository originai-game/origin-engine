import { Text, View, t } from "@origingame/engine/ui";
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
    <View class={`flex-row items-end gap-x-[0.857rem] text-fg compact:gap-x-[0.571rem]`} role="group" label={t("racing.pedals.name")}>
      <View class="flex-col items-center gap-y-[0.357rem]">
        <View
          class={`relative w-[0.857rem] h-[4.143rem] overflow-hidden bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38] compact:w-[0.643rem] compact:h-[2.857rem]`}
          role="meter"
          label={t("racing.pedals.throttleName")}
          valueMin={0}
          valueMax={100}
          valueNow={props.throttle <= 0 ? 0 : props.throttle >= 1 ? 100 : props.throttle * 100}
        >
          <View class="absolute bottom-0 left-0 w-full bg-[#55e39a]" style={{ height: `${(props.throttle <= 0 ? 0 : props.throttle >= 1 ? 1 : props.throttle) * 100}%` }} />
        </View>
        <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] text-stroke-3 text-stroke-bg compact:tracking-normal`}>{t("racing.pedals.throttle")}</Text>
      </View>
      <View class="flex-col items-center gap-y-[0.357rem]">
        <View
          class={`relative w-[0.857rem] h-[4.143rem] overflow-hidden bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38] compact:w-[0.643rem] compact:h-[2.857rem]`}
          role="meter"
          label={t("racing.pedals.brakeName")}
          valueMin={0}
          valueMax={100}
          valueNow={props.brake <= 0 ? 0 : props.brake >= 1 ? 100 : props.brake * 100}
        >
          <View class="absolute bottom-0 left-0 w-full bg-[#ff4d5e]" style={{ height: `${(props.brake <= 0 ? 0 : props.brake >= 1 ? 1 : props.brake) * 100}%` }} />
        </View>
        <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] text-stroke-3 text-stroke-bg compact:tracking-normal`}>{t("racing.pedals.brake")}</Text>
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
