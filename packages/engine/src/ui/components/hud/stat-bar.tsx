import { For, Icon, Show, Text, View, t, packClass } from "@origingame/engine/ui";
import "./stat-bar.en.strings.ts";

export interface StatBarProps {
  /** The value. */
  value: number;
  /** Its maximum. Default 1. */
  max?: number;
  /** What it measures (its colour): health, mana, stamina or xp; health turns warn and danger as it falls. Default health. */
  kind?: string;
  /** Value per block: a notch at each block's end (0: one bar). */
  segment?: number;
  /** Shield or armour on top of the value, a strip over the bar. */
  shield?: number;
  /** The most shield; 0 hides the strip. */
  shieldMax?: number;
  /** Healing on its way, a pale preview past the fill. */
  heal?: number;
  /** Damage that can still come back, drawn grey between the value and this. */
  recoverable?: number;
  /** Value before the latest loss, held as a trailing ghost behind the fill. Absent, the ghost rests at the fill and lags it on each loss. */
  ghost?: number;
  /** A short name at its start (HP); empty is none. */
  label?: string;
  /** An icon of the atlas before it; empty is none. */
  icon?: string;
  /** The value as text at its end. */
  showValue?: boolean;
  /** The value inside the track at its end, so the bar keeps its length. */
  valueInside?: boolean;
  /** The value as text instead of value / max. */
  valueText?: string;
  /** The fraction under which it is low (0.3). Default 0.3. */
  low?: number;
  /** The fraction under which it is critical (0.15). Default 0.15. */
  critical?: number;
  /** No HUD plate behind it. */
  bare?: boolean;
  /** What a screen reader calls it; empty is the label, else Health. */
  name?: string;
}

/**
 * A stat with its bar: the icon and label before a well that
 * fills to value in its kind's colour, the heal and recoverable shares
 * past it, a trailing ghost behind losses, the shield along its top,
 * a notch at each interior block boundary, and the figure at its end
 * (inside the track on valueInside). Health turns warn under low and
 * danger under critical. Read only.
 */
export function StatBar(props: StatBarProps) {
  const max = () => props.max ?? 1;
  const fraction = () => (max() > 0 ? Math.min(Math.max(props.value / max(), 0), 1) : 0);
  const ghostFraction = () => {
    if (max() <= 0) return 0;
    const g = Math.max(props.ghost ?? props.value, props.value);
    return Math.min(Math.max(g / max(), 0), 1);
  };
  const recoverableFraction = () => (max() > 0 ? Math.min(Math.max((props.recoverable ?? 0) / max(), fraction()), 1) : 0);
  const healFraction = () => (max() > 0 ? Math.min(Math.max((props.value + (props.heal ?? 0)) / max(), 0), 1) : 0);
  const text = () => props.valueText || t("component.stat-bar.value", { value: props.value, max: max() });
  const critical = () => (props.kind ?? "health") == "health" && fraction() <= (props.critical ?? 0.15);
  const low = () => (props.kind ?? "health") == "health" && fraction() <= (props.low ?? 0.3) && !critical();
  const cells = () => ((props.segment ?? 0) > 0 && max() > 0 ? Math.floor(max() / (props.segment ?? 1)) : 0);
  const block = () => (cells() > 0 && max() > 0 ? (props.segment ?? 1) / max() : 0);
  return (
    <View
      class={`group/stat flex-col gap-y-0.5 w-full text-fg data-mana:text-mana data-stamina:text-success data-xp:text-xp data-low:text-warn data-critical:text-danger`}
      role="meter"
      label={props.name || props.label || t("component.stat-bar.health")}
      valueMin={0}
      valueMax={max()}
      valueNow={props.value}
      valueText={text()}
      data-mana={(props.kind ?? "health") == "mana"}
      data-stamina={(props.kind ?? "health") == "stamina"}
      data-xp={(props.kind ?? "health") == "xp"}
      data-low={low()}
      data-critical={critical()}
      data-bare={props.bare}
    >
      <View class={`flex-row items-center gap-1 ${props.bare ? "" : packClass("Plate", "frame")} py-1 px-2`}>
        <Show when={props.icon}>
          <Icon class="size-4 shrink-0" name={props.icon} />
        </Show>
        <Show when={props.label}>
          <View class="bg-raised-pressed inset-ring-hairline inset-ring-border rounded-control py-0 px-1 shrink-0">
            <Text class="text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted text-shadow-tag">{props.label}</Text>
          </View>
        </Show>
        <View class={`relative min-w-0 flex-1 ${packClass("Bar", "frame")} h-4.5 ${critical() ? packClass("Bar", "frame", "low") : ""}`}>
          <Show when={(props.recoverable ?? 0) > props.value}>
            <View class="absolute left-0 top-0 h-full bg-fg-muted opacity-muted" style={{ width: `${recoverableFraction() * 100}%` }} />
          </Show>
          <View class={`absolute left-0 top-0 h-full ${packClass("Bar", "ghost")} ${packClass("Bar", "trail")}`} style={{ width: `${ghostFraction() * 100}%` }} />
          <Show when={(props.heal ?? 0) > 0}>
            <View class="absolute left-0 top-0 h-full bg-success opacity-soft" style={{ width: `${healFraction() * 100}%` }} />
          </Show>
          <View class={`absolute left-0 top-0 h-full bg-health inset-shadow-lit ${packClass("Bar", "fill")} group-data-mana/stat:bg-mana group-data-stamina/stat:bg-success group-data-xp/stat:bg-xp group-data-low/stat:bg-warn group-data-critical/stat:bg-critical`} style={{ width: `${fraction() * 100}%` }} />
          <Show when={(props.shieldMax ?? 0) > 0 && (props.shield ?? 0) > 0}>
            <View class="absolute left-0 top-0 h-17/50 bg-info" style={{ width: `${Math.min(Math.max((props.shield ?? 0) / (props.shieldMax ?? 1), 0), 1) * 100}%` }} />
          </Show>
          <Show when={cells() > 1}>
            <For each={Array.from({ length: Math.max(cells() - 1, 0) }, (_, i) => i)}>
              {(i) => <View class={`absolute top-0 bottom-0 -translate-x-full ${packClass("Bar", "tick")}`} style={{ left: `${(i() + 1) * block() * 100}%` }} />}
            </For>
          </Show>
          <Show when={props.showValue && props.valueInside}>
            <View class="absolute right-0 top-0 h-full flex-row items-center px-2 pointer-events-none">
              <Text class="text-xs font-bold tabular-nums whitespace-nowrap text-fg text-stroke-2 text-stroke-bg text-shadow-tag group-data-critical/stat:text-danger">{text()}</Text>
            </View>
          </Show>
        </View>
        <Show when={props.showValue && !props.valueInside}>
          <View class="bg-raised-pressed inset-ring-hairline inset-ring-border rounded-control py-0 px-1 shrink-0">
            <Text class="text-reading font-bold tabular-nums whitespace-nowrap text-fg text-shadow-tag group-data-critical/stat:text-danger">{text()}</Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StatBarProps>> = {
  default: {"value":72,"max":100,"kind":"health","segment":25,"shield":60,"shieldMax":100,"heal":12,"recoverable":0,"label":"HP","icon":"heart","showValue":true,"valueInside":false,"valueText":"","low":0.3,"critical":0.15,"bare":false,"name":""},
};
