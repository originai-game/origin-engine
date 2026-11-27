import { Icon, Show, Text, View, formatDuration, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./ability-slot.en.strings.ts";

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
 * swept dark over it and the seconds over that, nudging when it comes
 * ready. It draws the Ability component's pack slots, so it takes the
 * theme's size, shape, art and sweep.
 */
export function Cooldown(props: CooldownProps) {
  const cooling = () => props.remaining > 0;
  return (
    <View
      class={`relative ${packClass("Ability", "box")} ${packClass("Ability", "frame")} data-ready:animate-nudge`}
      role="button"
      onPress={(e) => props.onUse?.(e)}
      label={props.label}
      valueText={cooling() ? formatNumber(props.remaining, { decimals: 1 }) : t("component.ability-slot.ready")}
      disabled={cooling()}
      data-ready={!cooling()}
    >
      <View class={`absolute items-center justify-center ${packClass("Ability", "face")}`}>
        <Icon class={packClass("Ability", "icon")} name={props.icon} />
        <Show when={cooling()}>
          <View class={`absolute inset-0 ${packClass("Ability", "sweep")}`} wipe={Math.min(Math.max((props.total ?? 1) > 0 ? props.remaining / (props.total ?? 1) : 1, 0), 1)} wipeCcw />
        </Show>
      </View>
      <Show when={cooling()}>
        <View class="absolute inset-0 items-center justify-center pointer-events-none">
          <Text class={`whitespace-nowrap ${packClass("Ability", "count")}`}>
            {props.remaining < 10 ? formatNumber(props.remaining, { decimals: 1 }) : formatDuration(props.remaining)}
          </Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CooldownProps>> = {
  default: {"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"},
  ready: {"remaining":0,"total":8,"icon":"bolt","label":"Dash"},
};
