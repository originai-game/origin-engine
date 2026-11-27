import { For, Icon, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./star-bar.en.strings.ts";

export interface StarBarStar {
  id: string;
  /** Where it sits on the track, a fraction of max (0 to 1). */
  at: number;
}

export interface StarBarProps {
  /** The score. */
  value?: number;
  /** The bar's full score (its last threshold). Default 1. */
  max?: number;
  /** The thresholds: lit once the score reaches each. */
  stars?: ReadonlyArray<StarBarStar>;
  /** A caption over the score; empty is none. */
  label?: string;
  /** The score in words instead of the grouped number. */
  valueText?: string;
  /** One row: the score beside a shorter bar. */
  compact?: boolean;
}

/**
 * A score against its thresholds: the figure over a track that fills in
 * the style's gold, each threshold a star set on the track where it falls,
 * lit once the score reaches it. The stars are the style's rating stars,
 * the same as a level's or a results screen's. The style draws SVG stars;
 * here each is the atlas star icon in a seat, which keeps their shape and
 * colour but not the style's glow.
 */
export function StarBar(props: StarBarProps) {
  const max = () => props.max ?? 1;
  const stars = () => props.stars ?? [];
  const lit = (at: number) => (props.value ?? 0) >= at * max();
  return (
    <View
      class="group/starbar flex-col items-center gap-y-1 data-compact:flex-row"
      data-compact={props.compact === true}
      role="meter"
      label={props.label || t("component.star-bar.name")}
      valueMin={0}
      valueMax={max()}
      valueNow={props.value ?? 0}
    >
      <View class={`flex-col items-center py-0.5 px-2 ${packClass("Plate", "frame")}`}>
        <Show when={(props.label ?? "") !== "" && !props.compact}>
          <Text class="text-caption text-fg-muted">{props.label}</Text>
        </Show>
        <Text class="text-display-sm font-bold tabular-nums whitespace-nowrap text-fg group-data-compact/starbar:text-xl">{props.valueText || formatNumber(props.value ?? 0)}</Text>
      </View>
      <View class={`relative group-data-compact/starbar:w-35 w-50 group-data-compact/starbar:h-6 h-8`}>
        <View class="absolute inset-x-4 top-3 h-2 overflow-hidden rounded-capsule bg-bg-subtle group-data-compact/starbar:inset-x-3 group-data-compact/starbar:top-2.5 group-data-compact/starbar:h-1">
          <View class="h-full bg-gold" style={{ width: `${((props.value ?? 0) / max()) * 100}%` }} />
        </View>
        <For each={stars()} key={(star) => star.id}>
          {(star) => (
            <View
              class={`absolute top-0 size-8 rounded-full border bg-gold-fg flex-row items-center justify-center group-data-compact/starbar:size-6 ${lit(star().at) ? "border-gold data-lit:animate-beat" : "border-rim-strong"}`}
              data-lit={lit(star().at)}
              style={{ left: `calc(${star().at} * (100% - ${props.compact ? "1.5rem" : "2rem"}))` }}
            >
              <Icon class={`size-37/50 ${lit(star().at) ? "text-gold" : "text-fg-muted"}`} name="star" />
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StarBarProps>> = {
  default: {"value":2140,"max":3000,"stars":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"label":"Score","valueText":"","compact":false},
};
