import { For, Icon, Show, Text, View, t } from "@origingame/engine/ui";
import "./stat-bar.en.ogt";

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
 * A stat with its bar: the icon and label before a leaning well that
 * fills to value in its kind's colour, the heal and recoverable shares
 * past it, the shield along its top, a notch at each block's end, and the
 * figure at its end (inside the track on valueInside). Health turns warn
 * under low and danger under critical. Read only. The old style's lean
 * (skewX) and the blocks' mask have no TSX surface: the blocks are drawn
 * as notch marks, the lean and the figure's font-stretch are dropped, and
 * the ghost holds the fill's width instead of draining behind it.
 */
export function StatBar(props: StatBarProps) {
  const max = () => props.max ?? 1;
  const fraction = () => (max() > 0 ? Math.min(Math.max(props.value / max(), 0), 1) : 0);
  const text = () => props.valueText || t("component.stat-bar.value", { value: props.value, max: max() });
  const critical = () => (props.kind ?? "health") == "health" && fraction() <= (props.critical ?? 0.15);
  const low = () => (props.kind ?? "health") == "health" && fraction() <= (props.low ?? 0.3) && !critical();
  const cells = () => ((props.segment ?? 0) > 0 ? Math.floor(max() / (props.segment ?? 1)) : 0);
  const block = () => (cells() > 0 ? (props.segment ?? 1) / max() : 0);
  return (
    <View
      class={`group/stat flex-col gap-y-[0.143rem] w-full text-fg data-mana:text-mana data-stamina:text-success data-xp:text-xp data-low:text-warn data-critical:text-danger`}
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
      <View class={`flex-row items-center gap-1 ${props.bare ? "" : "border-rim bg-linear-to-b from-plate-hi to-plate-lo shadow-plate"} py-1 px-2`}>
        <Show when={props.icon}>
          <Icon class="size-4 shrink-0" name={props.icon} />
        </Show>
        <Show when={props.label}>
          <View class="bg-raised-pressed/88 shadow-[inset_0px_0px_0px_1px_#ffffff38] rounded-control py-0 px-1 shrink-0">
            <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg-muted text-shadow-2xs">{props.label}</Text>
          </View>
        </Show>
        <View class="relative min-w-0 flex-1 h-[1.286rem] overflow-hidden bg-track shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24,0px_0.143rem_0.286rem_#00000073] group-data-critical/stat:shadow-[0px_0px_0px_1px_#de2634,0px_0px_0.857rem_#de2634b3] group-data-critical/stat:animate-pulse">
          <View class="absolute left-0 top-0 h-full bg-fg-muted/80" style={{ width: `${Math.min(Math.max((props.recoverable ?? 0) / max(), fraction()), 1) * 100}%` }} />
          <View class="absolute left-0 top-0 h-full bg-linear-to-r from-gold/15 to-gold/85" style={{ width: `${fraction() * 100}%` }} />
          <Show when={(props.heal ?? 0) > 0}>
            <View class="absolute left-0 top-0 h-full bg-success/50" style={{ width: `${Math.min((props.value + (props.heal ?? 0)) / max(), 1) * 100}%` }} />
          </Show>
          <View class="absolute left-0 top-0 h-full bg-health shadow-[inset_0px_1px_0px_0px_#ffffff6b] transition-layout duration-180 ease-out group-data-mana/stat:bg-mana group-data-stamina/stat:bg-success group-data-xp/stat:bg-xp group-data-low/stat:bg-warn group-data-critical/stat:bg-critical" style={{ width: `${fraction() * 100}%` }} />
          <Show when={(props.shieldMax ?? 0) > 0 && (props.shield ?? 0) > 0}>
            <View class="absolute left-0 top-0 w-full h-[34%] bg-info" style={{ width: `${Math.min((props.shield ?? 0) / (props.shieldMax ?? 1), 1) * 100}%` }} />
          </Show>
          <Show when={cells() > 0}>
            <For each={Array.from({ length: cells() }, (_, i) => i)}>
              {(i) => <View class="absolute top-0 bottom-0 w-[0.8%] bg-glass/95 -translate-x-full" style={{ left: `${(i() + 1) * block() * 100}%` }} />}
            </For>
          </Show>
          <Show when={props.showValue && props.valueInside}>
            <View class="absolute right-0 top-0 h-full flex-row items-center px-2">
              <Text class="text-[14.88px] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs group-data-critical/stat:text-danger">{text()}</Text>
            </View>
          </Show>
        </View>
        <Show when={props.showValue && !props.valueInside}>
          <View class="bg-raised-pressed/88 shadow-[inset_0px_0px_0px_1px_#ffffff38] rounded-control py-0 px-1 shrink-0">
            <Text class="text-[14.88px] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs group-data-critical/stat:text-danger">{text()}</Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StatBarProps>> = {
  default: {"value":72,"max":100,"kind":"health","segment":25,"shield":60,"shieldMax":100,"heal":12,"recoverable":0,"label":"HP","icon":"heart","showValue":true,"valueInside":false,"valueText":"","low":0.3,"critical":0.15,"bare":false,"name":""},
};
