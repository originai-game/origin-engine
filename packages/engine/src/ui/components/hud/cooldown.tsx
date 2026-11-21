import { Icon, Show, Text, View, formatDuration, formatNumber, packClass, type UiEvent } from "@origingame/engine/ui";

export interface CooldownProps {
  /** Seconds left; 0 is ready. */
  remaining: number;
  /** The whole cooldown in seconds. Default 1. */
  total?: number;
  /** An icon of the atlas under the sweep. */
  icon: string;
  /** Its accessible name (Dash). */
  label: string;
  /** Use. */
  onUse?: (e: UiEvent) => void;
}

/**
 * A scene cell a game lays in a strip: its icon, the share still to run
 * darkened over it and the seconds reading on their halo, popping when it
 * comes ready. The sweep is a nine-disc wipe running counterclockwise
 * from the top, as the .svelte's conic mask did.
 */
export function Cooldown(props: CooldownProps) {
  const cooling = () => props.remaining > 0;
  return (
    <View
      class={`flex-row items-center justify-center gap-1 group/cooldown size-12 overflow-hidden rounded-button border-rim-strong bg-linear-to-b from-plate-hi to-plate-lo shadow-[0px_0px_0px_1px_#00000073,inset_0px_1px_0px_0px_#ffffff14,0px_6px_18px_-6px_#00000099] active:bg-none active:bg-tile-pressed active:scale-96 data-ready:animate-pop ${packClass("Slot", "frame")}`}
      role="button"
      onPress={(e) => props.onUse?.(e)}
      label={props.label}
      valueText={cooling() ? formatNumber(props.remaining) : "Ready"}
      disabled={cooling()}
      data-ready={!cooling()}
    >
      <View class="relative size-full">
        <View class="absolute inset-0 flex-row items-center justify-center">
          <Icon class="size-8 text-fg" name={props.icon} />
        </View>
        <Show when={cooling()}>
          <View class="absolute inset-0 nine-disc bg-shade-deep" wipe={Math.min(props.remaining / (props.total ?? 1), 1)} wipeCcw />
          <View class="absolute inset-0 flex-row items-center justify-center">
            <Text class="text-[20.64px] tabular-nums text-center text-fg text-shadow-xs">
              {props.remaining < 10 ? formatNumber(props.remaining) : formatDuration(props.remaining)}
            </Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CooldownProps>> = {
  default: {"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"},
};
