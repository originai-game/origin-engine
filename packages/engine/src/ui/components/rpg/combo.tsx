import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgComboProps {
  /** The hits in the chain; 0 hides the counter. */
  count: number;
  /** The word after the figure as the game words it; empty takes the component's Hits. */
  word: string;
  /** The share of the chain's window still to run, 0 to 1 (the decay bar); negative hides the bar. */
  remaining?: number;
  /** The damage multiplier the chain gives (Damage ×1.6); 0 or 1 hides it. */
  multiplier?: number;
}

/**
 * The combo counter beside the fight: the hits as a large slanted figure
 * with its word, the window still to run as a decay bar under it and the
 * damage multiplier the chain gives. The game owns the chain; read only.
 */
export function RpgCombo(props: RpgComboProps) {
  return (
    <View
      class={`flex-col items-end max-w-full text-fg text-stroke-3 text-stroke-bg`}
      role="status"
      label={t("rpg.combo.hits", { count: props.count })}
    >
      <Show when={props.count > 0}>
        <View class={`flex-row items-baseline justify-end max-w-full min-w-0 gap-1`}>
          <Text class={`shrink-0 text-8xl leading-flat font-bold italic tabular-nums whitespace-nowrap compact:text-4xl animate-pop`}>
            {formatNumber(props.count)}
          </Text>
          <Text
            class={`min-w-0 text-title font-bold italic tracking-label uppercase whitespace-nowrap overflow-hidden compact:text-caption`}
          >
            {props.word !== "" ? props.word : t("rpg.combo.word")}
          </Text>
        </View>
        <Show when={(props.remaining ?? -1) >= 0}>
          <View
            class={`w-40 max-w-full h-1.5 mt-1.5 overflow-hidden -skew-x-22 bg-track shadow-raise ring ring-rim compact:w-27.5 compact:h-1.25 compact:mt-1`}
          >
            <View
              class={`h-full bg-hud-accent bg-linear-to-b from-sheen to-transparent to-42% shadow-glow`}
              style={{ width: `${Math.min((props.remaining ?? -1) >= 1 ? 1 : props.remaining ?? 0, 1) * 100}%` }}
            />
          </View>
        </Show>
        <Show when={(props.multiplier ?? 0) > 1}>
          <Text
            class={`mt-1.5 text-xs font-semibold tracking-label uppercase text-hud-accent-ink whitespace-nowrap compact:mt-0.75 compact:tracking-normal`}
          >
            {t("rpg.combo.bonus", { value: props.multiplier ?? 0 })}
          </Text>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgComboProps>> = {
  default: {"count":27,"word":"","remaining":0.6,"multiplier":1.6},
  fading: {"count":4,"word":"","remaining":0.08,"multiplier":1},
  empty: {"count":0,"word":"","remaining":-1,"multiplier":0},
  overflow: {"count":1234567,"word":"Uninterrupted strikes","remaining":1,"multiplier":12.5},
};
