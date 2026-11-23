import { For, Icon, View } from "@origingame/engine/ui";

export interface StarRatingProps {
  /** A keyed collection of { id, index (0 first), lit }: one star each; a lit star is earned and drawn filled. */
  stars: ReadonlyArray<{ id: string; index: number; lit: boolean }>;
  /** What it says to a screen reader (2 of 3 stars). */
  label: string;
  /** md (the default): 32 px stars in a line. lg: a result's grade, 78 px stars (52 on a phone) with the middle of three raised and larger, as the puzzle summary draws them. */
  size?: "md" | "lg";
  /** Classes after the row's own (a placement, a margin). */
  class?: string | undefined;
}

/**
 * A row of rating stars, each lit once it is earned. The style draws the
 * stars as SVG glyphs (an outlined empty star, a gold one with a lit edge
 * and a glow); the engine draws no vector shapes, so each is the atlas
 * star icon, one tint lit and a dim one unearned, landing with the pop
 * entrance (the style's land, without its overshoot) delayed by the
 * star's index.
 */
export function StarRating(props: StarRatingProps) {
  const lg = () => props.size === "lg";
  const centre = (index: number) => lg() && props.stars.length === 3 && index === 1;
  return (
    <View class={`flex-row text-fg p-0 gap-1 disabled:text-fg-subtle data-size-lg:items-end data-size-lg:gap-2 ${props.class ?? ""}`} role="image" label={props.label} data-size-lg={lg()}>
      <For each={props.stars}>
        {(s) => (
        <View class={`size-8 shrink data-size-lg:size-19.5 data-size-lg:compact:size-13 data-centre:size-25 data-centre:mb-3 data-centre:compact:size-17 data-centre:compact:mb-2 ${s().lit ? "data-lit:animate-pop" : ""}`} data-lit={s().lit} data-size-lg={lg() && !centre(s().index)} data-centre={centre(s().index)} style={{ animationDelay: `${(120 * (s().index))}ms` }}>
          <Icon class={`inset-0 size-full ${s().lit ? "text-gold" : lg() ? "text-fg-subtle" : "text-fg"}`} name="star" />
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<StarRatingProps>> = {
  default: {"label":"2 of 3 stars","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":false}]},
  none: {"label":"0 of 3 stars","stars":[{"id":"1","index":0,"lit":false},{"id":"2","index":1,"lit":false},{"id":"3","index":2,"lit":false}]},
  large: {"label":"2 of 3 stars","size":"lg","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":false}]},
  all: {"label":"3 of 3 stars","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":true}]},
};
