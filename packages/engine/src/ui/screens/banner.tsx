import {Text, View, packClass} from "@origingame/engine/ui";

export interface BannerProps {
  /** Its headline (Level up!). */
  title: string;
  /** A line under it; empty: none. */
  subtitle: string;
}

/**
 * A call across the scene with no plate: the headline in large HUD type
 * (white, one stroke under the fill and a soft shadow), a lit gold rule
 * under it and the line beneath in quieter HUD type. It rises in with the
 * reward sound; under reduced motion it fades in place.
 */
export function Banner(props: BannerProps) {
  return (
    <View class={`text-fg items-center text-center py-0 px-4 gap-1 transition duration-420 ease-out starting:scale-80 starting:opacity-0 ${packClass("Plate", "frame")}`} role="status">
      <Text class={`font-display text-[3.857rem] font-bold leading-tight tracking-[-0.01em] text-fg text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg/55 max-w-140 text-balance m-0 compact:text-[2.429rem]`}>
        {props.title}
      </Text>
      <View class={`h-0.75`} />
      <Text class={`text-[1.071rem] font-semibold tracking-[0.14em] uppercase text-gold text-balance text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg/55 m-0 data-hidden:hidden`} data-hidden={props.subtitle === ""}>
        {props.subtitle}
      </Text>
    </View>
  );
}

export const fixtures: Record<string, Partial<BannerProps>> = {
  default: {"title":"Level up!","subtitle":"Level 12"},
};
