import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./orb-meter.en.ogt";

export interface OrbMeterProps {
  /** The value. */
  value: number;
  /** Its maximum. Default 1. */
  max?: number;
  /** Its liquid's colour: health or mana. */
  kind?: string;
  /** The value in the middle. */
  showValue?: boolean;
  /** A caption under it; empty is none. */
  label?: string;
  /** What a screen reader calls it; empty is the label, else Health. */
  name?: string;
}

/**
 * A glass vessel in a bezel, over the scene: a bezel with its light and
 * shade, a socket, a dark well in its liquid's hue with the liquid
 * standing to the value, a surface line, a gleam blob riding the surface
 * and the figure in the middle. The style's SVG waves, glare arcs and
 * ornament have no TSX surface: the liquid is a flat fill with a surface
 * line, the glare a white blob, the ornament dropped (the default pack
 * draws none). The packs' --orb-meter-* tokens are resolved to the dark
 * scheme's values here.
 */
export function OrbMeter(props: OrbMeterProps) {
  const max = () => Math.max(props.max ?? 1, 0);
  const share = () => (max() <= 0 ? 0 : Math.min(Math.max(props.value, 0) / max(), 1));
  return (
    <View
      class={`group/orb flex-col items-center gap-y-[0.143rem]`}
      role="meter"
      label={(props.name ?? "") !== "" ? props.name : (props.label ?? "") !== "" ? props.label : t("component.orb-meter.health")}
      valueMin={0}
      valueMax={props.max ?? 1}
      valueNow={props.value}
      data-mana={(props.kind ?? "") === "mana"}
    >
      <View class="relative size-[8rem]">
        <View class="absolute inset-0 rounded-full bg-[#8892a8] shadow-[0px_0px_0px_1px_#292c32,0px_0.143rem_0.286rem_#00000073]">
          <View class="absolute inset-0 rounded-full bg-linear-to-b from-white/60 to-transparent to-45%" />
          <View class="absolute inset-0 rounded-full bg-linear-to-t from-black/60 to-transparent to-45%" />
        </View>
        <View class="absolute inset-[0.429rem] rounded-full bg-[#181a1e]" />
        <View class="absolute inset-[0.643rem] rounded-full overflow-hidden bg-[#0c3215] shadow-[inset_0px_0px_1.429rem_#0000009e] group-data-mana/orb:bg-[#0f253d]">
          <View class="absolute inset-x-0 bottom-0 border-t-[1.4px] border-t-[#a2eab4] bg-[#30d158] group-data-mana/orb:border-t-[#a9d2ff] group-data-mana/orb:bg-[#409cff]" style={{ height: `${share() * 100}%` }} />
          <View class="absolute left-[13.4%] size-[73.2%] rounded-full bg-white/15" style={{ top: `${Math.max(0, Math.min(1, 1.049 - share())) * 100}%` }} />
          <View class="absolute inset-0 bg-linear-to-b from-transparent from-35% to-black/40" />
        </View>
        <View class="absolute inset-[0.643rem] rounded-full shadow-[inset_0px_0px_0px_1px_#ffffff47] pointer-events-none" />
        <Show when={props.showValue}>
          <View class="absolute inset-0 flex-row items-center justify-center">
            <Text class="text-[27.36px] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{formatNumber(props.value)}</Text>
          </View>
        </Show>
      </View>
      <Show when={props.label}>
        <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg text-shadow-xs">{props.label}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<OrbMeterProps>> = {
  default: {"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""},
  mana: {"value":220,"max":400,"kind":"mana","showValue":true,"label":"Arcana","name":""},
  empty: {"value":0,"max":500,"kind":"health","showValue":false,"label":"","name":""},
  full: {"value":500,"max":500,"kind":"health","showValue":true,"label":"Life","name":""},
};
