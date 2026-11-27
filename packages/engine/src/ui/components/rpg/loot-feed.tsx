import type { Texture } from "@origingame/engine";
import { For, View, t } from "@origingame/engine/ui";
import { HudLootEntry } from "../hud/loot-entry";
import "./rpg.en.strings.ts";

export interface RpgLootFeedProps {
  /** What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows. */
  items?: ReadonlyArray<{ id: string; name: string; icon: Texture | 0; glyph: string; rarity: string; label: string; quantity: number; currency: boolean; fading: boolean }>;
}

/**
 * The loot feed low on the screen's side: a column of HudLootEntry
 * blocks, newest first, each an icon, the item's name, its rarity on a
 * tag and the count gained on a dark glass with its edge tinted by
 * rarity. Read only.
 */
export function RpgLootFeed(props: RpgLootFeedProps) {
  return (
    <View class={`flex-col items-start gap-y-1.25 max-w-full text-fg`} role="log" label={t("rpg.lootfeed.name")}>
      <For each={props.items ?? []} key={(p) => p.id}>
        {(p, i) => (
        <View class={`max-w-full min-w-0 shrink-0 compact:data-later:hidden transition duration-base ease-out starting:opacity-0 starting:translate-y-2 exit:opacity-0`} data-later={i() >= 1}>
          <HudLootEntry name={p().name} icon={p().icon} glyph={p().glyph} rarity={p().rarity} label={p().label} quantity={p().quantity} currency={p().currency} fading={p().fading} />
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLootFeedProps>> = {
  default: {"items":[{"id":"blade","name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false},{"id":"shard","name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true}]},
  rare: {"items":[{"id":"ring","name":"Tidewatch Ring","icon":0,"glyph":"gem","rarity":"rare","label":"Rare","quantity":0,"currency":false,"fading":false}]},
  overflow: {"items":[{"id":"long","name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false}]},
};
