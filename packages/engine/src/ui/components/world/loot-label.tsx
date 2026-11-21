import { For, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface LootTier {
  id: string;
  rarity?: string;
}

export interface LootLabelProps {
  /** The item, by its bits (0: use at). */
  entity?: number;
  /** A world point it lies at without an entity (x y z). */
  at?: string;
  /** World units above it the label stands. */
  height?: number;
  /** The item's name, in its tier's ink; a long one scrolls on one line. */
  name?: string;
  /** A stack's size, shown as ×n from 2. */
  count?: number;
  /** Its tier: common (empty), uncommon, rare, epic or legendary: the strip on the chip's leading edge, the name's ink and the glow on the ground; epic and legendary raise a beam, a legendary wears a gold wash and rim and shimmers every 2.4 s. */
  rarity?: string;
  /** Aimed at or under the pointer (the chip takes :hover too): it lifts 1.06 times, its rim goes all round and the pick-up key shows. */
  hovered?: boolean;
  /** The input action whose bound key shows on a hovered chip (og.input.defineAction); empty shows none. */
  action?: string;
  /** A chip standing for this many more items the game folded into one (+n), with a dot per tier in tiers; 0 is an item's own chip. */
  more?: number;
  /** The folded items' tiers, keyed rows of { id, rarity }, a dot each on a +n chip. */
  tiers?: ReadonlyArray<LootTier>;
  /** The chip is pressed: the game owns what is picked up. */
  onPick?: (e: UiEvent) => void;
}

/**
 * An item on the ground: over it a dark chip with the tier's strip on its
 * leading edge and the name in the tier's ink (a stack's count after it);
 * epic and legendary raise a beam. Reports pick (the chip); the game owns
 * what is picked up. The old ground pool (the tier's glow where the item
 * lies, a `<ground>` plane) has no TSX surface and is dropped, as are the
 * old anchor's avoidance and max distance; labels stack clear of each
 * other and fade behind drawn geometry (the anchor's declutter and
 * occlusion).
 * The hovered chip's pick-up key (the action's bound glyph) has no TSX
 * surface and is dropped; the chip keeps the action's name as a state. A
 * long name wraps without breaking.
 */
export function LootLabel(props: LootLabelProps) {
  const rarity = () => ((props.rarity ?? "") == "" ? "common" : (props.rarity ?? ""));
  const name = () => props.name ?? "";
  const action = () => props.action ?? "";
  const more = () => props.more ?? 0;
  const point = () => ((props.at ?? "0 0 0").split(/\s+/).map(Number) as [number, number, number]);
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : point()}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      occlusion="fade"
      class="-translate-x-1/2 group flex-col items-center"
      data-common={rarity() === "common"}
      data-uncommon={rarity() === "uncommon"}
      data-rare={rarity() === "rare"}
      data-epic={rarity() === "epic"}
      data-legendary={rarity() === "legendary"}
    >
      <View
        class={`group/chip relative flex-row items-center gap-x-1.5 min-w-0 max-w-full shrink-0 py-0.75 pr-2.25 pl-2.75 text-[0.964rem] leading-[1.2] font-semibold whitespace-nowrap nine-chip-cut bg-raised-pressed/88 animate-pop hover:scale-106 hover:bg-fg/96 group-data-uncommon:text-uncommon group-data-rare:text-rare group-data-epic:text-epic group-data-legendary:font-bold group-data-legendary:text-gold group-data-legendary:bg-tile/90 group-data-hovered:scale-106 group-data-hovered:bg-fg/96 text-fg ${packClass("Chip", "frame")}`}
        data-more={more() > 0}
        data-hovered={props.hovered}
        role="button"
        label={more() > 0 ? t("world.loot.moreName", { n: more() }) : name()}
        onPress={(e) => props.onPick?.(e)}
      >
        <View class="absolute left-0 top-0 bottom-0 w-0.75 bg-fg/45 group-data-uncommon:bg-uncommon group-data-rare:bg-rare group-data-epic:bg-epic group-data-legendary:bg-gold" />
        <Show when={more() > 0}>
          <Text class="shrink-0 font-bold tabular-nums">{t("world.loot.more", { n: more() })}</Text>
          <For each={props.tiers ?? []} key={(d) => d.id}>
            {(d) => <View class="shrink-0 size-2 rotate-45 bg-fg/45 data-rarity-uncommon:bg-uncommon data-rarity-rare:bg-rare data-rarity-epic:bg-epic data-rarity-legendary:bg-gold" data-rarity-uncommon={d().rarity === "uncommon"} data-rarity-rare={d().rarity === "rare"} data-rarity-epic={d().rarity === "epic"} data-rarity-legendary={d().rarity === "legendary"} />}
          </For>
        </Show>
        <Show when={more() === 0}>
          <Text class="shrink max-w-47.5 min-w-0 truncate opacity-90 group-data-uncommon:text-uncommon group-data-rare:text-rare group-data-epic:text-epic group-data-legendary:text-gold">{name()}</Text>
          <Show when={(props.count ?? 0) > 1}>
            <Text class="shrink-0 font-bold tabular-nums">{t("world.loot.count", { count: props.count ?? 0 })}</Text>
          </Show>
          <Show when={rarity() === "legendary"}>
            <View class="absolute left-0 top-0 bottom-0 w-3/10 bg-linear-to-r from-transparent via-gold-hi/45 to-transparent animate-shimmer" />
          </Show>
        </Show>
      </View>
      <Show when={more() === 0 && (rarity() === "epic" || rarity() === "legendary")}>
        <View class="shrink-0 w-0.75 h-11 bg-epic shadow-[0px_0px_1.143rem_#c86ff5] group-data-legendary:bg-gold group-data-legendary:shadow-[0px_0px_1.143rem_#ffd23f] opacity-75" />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<LootLabelProps>> = {
  default: {"entity":0,"at":"0 0 0","height":0,"name":"Ember Blade","count":1,"rarity":"epic","hovered":false,"action":"","more":0,"tiers":[]},
  common: {"name":"Rusted Buckle","rarity":""},
  stack: {"name":"Gold","count":214,"rarity":""},
  uncommon: {"name":"Wolfhide Bracers","rarity":"uncommon"},
  rare: {"name":"Stormglass Pendant","rarity":"rare"},
  legendary: {"name":"Crown of the Ninefold King","rarity":"legendary"},
  hovered: {"hovered":true,"action":"pick"},
  more: {"name":"","more":6,"rarity":"","tiers":[{"id":"a","rarity":"rare"},{"id":"b","rarity":"uncommon"},{"id":"c","rarity":"uncommon"},{"id":"d","rarity":""}]},
  overflow: {"name":"Blade of the Ninefold King’s Last Oath, Reforged in Ember","count":1250,"rarity":"legendary"},
};
