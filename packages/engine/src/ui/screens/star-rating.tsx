import { For, Icon, View } from "@origingame/engine/ui";

export interface StarRatingProps {
  /** A keyed collection of { id, index (0 first), lit }: one star each; a lit star is earned and drawn filled. */
  stars: ReadonlyArray<{ id: string; index: number; lit: boolean }>;
  /** What it says to a screen reader (2 of 3 stars). */
  label: string;
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
  return (
    <View class={`text-fg p-0 gap-1 disabled:text-fg-subtle`} role="image" label={props.label}>
      <For each={props.stars}>
        {(s) => (
        <View class={`size-8 shrink ${s().lit ? "data-lit:animate-pop" : ""}`} data-lit={s().lit} style={{ animationDelay: `${(120 * (s().index))}ms` }}>
          <Icon class={`inset-0 size-full ${s().lit ? "text-[#ffcf5a]" : "text-white/42"}`} name="star" />
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<StarRatingProps>> = {
  default: {"label":"2 of 3 stars","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":false}]},
  none: {"label":"0 of 3 stars","stars":[{"id":"1","index":0,"lit":false},{"id":"2","index":1,"lit":false},{"id":"3","index":2,"lit":false}]},
  all: {"label":"3 of 3 stars","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":true}]},
};
