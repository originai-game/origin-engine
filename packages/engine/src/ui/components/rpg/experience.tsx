import { packClass, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgExperienceProps {
  /** The level now; 0 hides the caption's level. */
  level: number;
  /** The share of the way to the next level, 0 to 1. */
  value: number;
}

/**
 * The experience line across the screen's foot: a thin violet line lit to
 * the share of the way to the next level, with the level and that share
 * in small capitals over its start, inset from the screen's edge as the
 * HUD is. Read only.
 */
export function RpgExperience(props: RpgExperienceProps) {
  const share = (v: number) => `${formatNumber(v * 100, { decimals: 0 })}%`;
  return (
    <View
      class={`flex-col w-full gap-y-2 text-fg nine-plate-cut bg-[#1e2637e5] p-2 ${packClass("Plate", "frame")}`}
      role="meter"
      label={t("rpg.xp.name")}
      valueMin={0}
      valueMax={100}
      valueNow={props.value <= 0 ? 0 : props.value >= 1 ? 100 : props.value * 100}
    >
      <Text
        class={`self-start max-w-full px-[1.714rem] text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ffffffc7] whitespace-nowrap overflow-hidden text-stroke-3 text-stroke-bg compact:px-3 compact:tracking-normal`}
      >
        {props.level > 0
          ? t("rpg.xp.caption", { level: props.level, share: share(props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value) })
          : t("rpg.xp.share", { share: share(props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value) })}
      </Text>
      <View class={`w-full h-[0.429rem] bg-[#04060cb2] compact:h-[0.286rem]`}>
        <View
          class={`h-full bg-linear-to-r from-[#8a6bff] to-[#d7b8ff] shadow-[0px_0px_10px_#a98bff]`}
          style={{ width: `${(props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value) * 100}%` }}
        />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgExperienceProps>> = {
  default: {"level":42,"value":0.62},
  empty: {"level":1,"value":0},
  full: {"level":99,"value":1},
};
