import { For, Show, Text, View, t, packClass } from "@origingame/engine/ui";
import "./bar.en.ogt";

export interface HudBarProps {
  /** The value now. */
  value: number;
  /** Its most; 0 draws the bar empty. Default 1. */
  max?: number;
  /** What it measures, which is its fill: health, mana, stamina, shield or xp. Default health. */
  kind?: string;
  /** The value before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none). */
  ghost?: number;
  /** Value per cell (one dodge's cost): the bar is cut into cells; 0 draws it whole. */
  segment?: number;
  /** Low: the fill turns to danger and the rim glows. */
  low?: boolean;
  /** Its figure, value over max, upright over the bar; for a bar thick enough to hold a line of text. */
  showValue?: boolean;
  /** What a screen reader calls it; empty is its kind (Health, Mana, Stamina, Experience). */
  name?: string;
}

/**
 * One bar of a HUD, a block a plate is built from: it fills to value over
 * max in its kind's colour over a dark well with a lit rim; a loss leaves
 * a warm ghost that drains after the fill, a low bar turns to danger,
 * segment cuts it into cells, and showValue stands its figure over it. It
 * is as wide as its place; a plate sizes the place. Read only. Its well,
 * fill colours, ghost, notches and figure are the Bar component's pack
 * slots, so a theme restyles every HUD bar at once.
 */
export function HudBar(props: HudBarProps) {
  const value = () => Math.min(Math.max(props.value, 0), Math.max(props.max ?? 1, 0));
  // The fill is value over max, as the original's --value was.
  const share = () => ((props.max ?? 1) > 0 ? value() / (props.max ?? 1) : 0);
  const ghost = () => {
    const max = props.max ?? 1;
    // It rests at the fill and lags it on a loss (Bar.trail); a game's ghost holds it longer.
    const g = Math.max(props.ghost ?? 0, props.value);
    return max <= 0 ? 0 : Math.min(Math.max(g, 0) / max, 1);
  };
  const cells = () => {
    const max = props.max ?? 1;
    return (props.segment ?? 0) > 0 && max > 0 ? max / (props.segment ?? 1) : 1;
  };
  return (
    <View
      class={`group/bar relative w-full ${packClass("Bar", "frame")} ${props.low ? packClass("Bar", "frame", "low") : ""}`}
      role="meter"
      label={props.name != "" && props.name != undefined ? props.name : (props.kind ?? "health") == "mana" ? t("hud.bar.mana") : (props.kind ?? "health") == "stamina" ? t("hud.bar.stamina") : (props.kind ?? "health") == "shield" ? t("hud.bar.shield") : (props.kind ?? "health") == "xp" ? t("hud.bar.xp") : t("hud.bar.health")}
      valueMin={0}
      valueMax={(props.max ?? 1) > 0 ? props.max : 1}
      valueNow={props.value <= 0 ? 0 : (props.max ?? 1) > 0 && props.value >= (props.max ?? 1) ? props.max : props.value}
      data-mana={(props.kind ?? "health") == "mana"}
      data-stamina={(props.kind ?? "health") == "stamina"}
      data-shield={(props.kind ?? "health") == "shield"}
      data-xp={(props.kind ?? "health") == "xp"}
      data-low={props.low}
    >
      <View class={`absolute left-0 top-0 h-full ${packClass("Bar", "ghost")} ${packClass("Bar", "trail")}`} style={{ width: `${ghost() * 100}%` }} />
      <View class={`${packClass("Bar", "fill")} absolute left-0 top-0 h-full ${packClass("Bar", "tone")} ${props.low ? packClass("Bar", "tone", "low") : (props.kind ?? "health") !== "health" ? packClass("Bar", "tone", props.kind ?? "health") : ""}`} style={{ width: `${share() * 100}%` }} />
      <Show when={(props.segment ?? 0) > 0 && (props.max ?? 1) > 0}>
        <For each={Array.from({ length: Math.max(Math.ceil(cells()) - 1, 0) }, (_, i) => i)}>
          {(i) => <View class={`absolute top-0 bottom-0 -translate-x-full ${packClass("Bar", "tick")}`} style={{ left: `${((i() + 1) / cells()) * 100}%` }} />}
        </For>
      </Show>
      <Show when={props.showValue}>
        <View class="absolute inset-0 flex-row items-center justify-center px-1 overflow-hidden">
          <Text class={`whitespace-nowrap ${packClass("Bar", "value")}`}>{t("hud.bar.figure", { now: props.value, max: props.max ?? 1 })}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudBarProps>> = {
  default: {"value":2184,"max":3420,"kind":"health","ghost":2668,"segment":0,"low":false,"showValue":false,"name":""},
  mana: {"value":72,"max":100,"kind":"mana","ghost":0,"segment":0,"low":false,"showValue":false,"name":""},
  cells: {"value":5,"max":8,"kind":"stamina","ghost":0,"segment":1,"low":false,"showValue":false,"name":""},
  shield: {"value":60,"max":100,"kind":"shield","ghost":0,"segment":25,"low":false,"showValue":false,"name":""},
  figure: {"value":62,"max":72,"kind":"health","ghost":0,"segment":0,"low":false,"showValue":true,"name":""},
  low: {"value":42,"max":3420,"kind":"health","ghost":300,"segment":0,"low":true,"showValue":false,"name":""},
  empty: {"value":0,"max":0,"kind":"health","ghost":0,"segment":0,"low":false,"showValue":false,"name":""},
  overflow: {"value":999999,"max":10,"kind":"xp","ghost":2000000,"segment":3,"low":false,"showValue":false,"name":"Experience towards the next level of the Ninefold Gate"},
};
