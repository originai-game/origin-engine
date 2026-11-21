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
      class={`group/orb flex-col items-center gap-y-0.5`}
      role="meter"
      label={(props.name ?? "") !== "" ? props.name : (props.label ?? "") !== "" ? props.label : t("component.orb-meter.health")}
      valueMin={0}
      valueMax={props.max ?? 1}
      valueNow={props.value}
      data-mana={(props.kind ?? "") === "mana"}
    >
      <View class="relative size-28">
        <View class="absolute inset-0 rounded-full bg-rim-strong shadow-plate ring ring-key-fg">
          <View class="absolute inset-0 rounded-full bg-linear-to-b from-sheen to-transparent to-45%" />
          <View class="absolute inset-0 rounded-full bg-linear-to-t from-shade-deep to-transparent to-45%" />
        </View>
        <View class="absolute inset-1.5 rounded-full bg-surface" />
        <View class="absolute inset-2.25 rounded-full overflow-hidden bg-boon-fg inset-shadow-sunk group-data-mana/orb:bg-raised-pressed">
          <View class="absolute inset-x-0 bottom-0 border-t border-t-boon bg-health group-data-mana/orb:border-t-mana group-data-mana/orb:bg-mana" style={{ height: `${share() * 100}%` }} />
          <View class="absolute left-13/100 size-73/100 rounded-full bg-sheen" style={{ top: `${Math.max(0, Math.min(1, 1.049 - share())) * 100}%` }} />
          <View class="absolute inset-0 bg-linear-to-b from-transparent from-35% to-shade-deep" />
        </View>
        <View class="absolute inset-2.25 rounded-full inset-ring-1 inset-ring-rim pointer-events-none" />
        <Show when={props.showValue}>
          <View class="absolute inset-0 flex-row items-center justify-center">
            <Text class="text-display-sm font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{formatNumber(props.value)}</Text>
          </View>
        </Show>
      </View>
      <Show when={props.label}>
        <Text class="text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg text-shadow-xs">{props.label}</Text>
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
