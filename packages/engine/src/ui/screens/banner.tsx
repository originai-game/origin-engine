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
    <View class={`text-fg items-center text-center py-0 px-4 gap-1 transition duration-420 ease-out starting:scale-80 starting:opacity-0 ${packClass("Plate", "frame")}`} role="status" sound="reward">
      <Text class={`font-display text-7xl font-bold leading-tight tracking-tight text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg max-w-140 text-balance m-0 compact:text-display-md`}>
        {props.title}
      </Text>
      <View class={`h-0.75`} />
      <Text class={`text-reading font-semibold tracking-label uppercase text-gold text-balance text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg m-0 data-hidden:hidden`} data-hidden={props.subtitle === ""}>
        {props.subtitle}
      </Text>
    </View>
  );
}

export const fixtures: Record<string, Partial<BannerProps>> = {
  default: {"title":"Level up!","subtitle":"Level 12"},
};
