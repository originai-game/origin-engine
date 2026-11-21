import { For, Icon, Image, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./loot-feed.en.ogt";
import "../hud/loot-entry.en.ogt";

export interface RpgLootFeedProps {
  /** What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows. */
  items?: ReadonlyArray<{ id: string; name: string; icon: number; glyph: string; rarity: string; label: string; quantity: number; currency: boolean; fading: boolean }>;
}

/**
 * The loot feed low on the screen's side: a column of HudLootEntry blocks
 * (inlined: hud/loot-entry.svelte is not ported yet), newest first, each
 * an icon, the item's name, its rarity on a tag and the count gained on a
 * dark glass with its edge tinted by rarity. Read only.
 */
export function RpgLootFeed(props: RpgLootFeedProps) {
  return (
    <View class={`flex-col items-start gap-y-[0.357rem] max-w-full text-fg`} role="log" label={t("rpg.lootfeed.name")}>
      <For each={props.items ?? []} key={(p) => p.id}>
        {(p, i) => (
        <View class={`max-w-full min-w-0 shrink-0 compact:data-later:hidden`} data-later={i() >= 1}>
          <View class={`group/loot flex-row items-center max-w-full min-w-0 pr-3 bg-linear-to-r from-[#0d1526db] to-[#0d152659] data-fading:opacity-55 transition duration-240 ease-out starting:opacity-0 starting:translate-y-2 exit:opacity-0`} role="group"
            label={p().quantity > 0 ? t("hud.loot-entry.row", { name: p().name, quantity: p().currency ? t("hud.loot-entry.gain", { quantity: formatNumber(p().quantity) }) : t("hud.loot-entry.count", { quantity: formatNumber(p().quantity) }) }) : p().name}
            data-fading={p().fading} data-rarity={p().rarity}
            data-uncommon={p().rarity === "uncommon"} data-rare={p().rarity === "rare"} data-epic={p().rarity === "epic"} data-legendary={p().rarity === "legendary"}>
            <View class={`shrink self-stretch w-[0.214rem] mr-1 bg-[#ffffff99] group-data-uncommon/loot:bg-uncommon group-data-rare/loot:bg-rare group-data-epic/loot:bg-epic group-data-legendary/loot:bg-gold`} />
            <View class="relative shrink my-[0.214rem] size-[1.429rem] compact:size-[1.286rem] overflow-hidden bg-[#060910e0] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none">
              <Show when={p().icon} fallback={
                  <Show when={p().glyph}>
                    <Icon class="absolute inset-0 size-full text-fg" name={p().glyph} />
                  </Show>
              }>
                <Image class="absolute inset-0 size-full" src={String(p().icon)} />
              </Show>
            </View>
            <Text class={`min-w-0 ml-1 text-[1rem] font-semibold whitespace-nowrap overflow-hidden text-fg text-shadow-xs text-shadow-bg/55 group-data-epic/loot:text-epic group-data-legendary/loot:text-gold compact:text-[0.929rem]`}>
              {p().name}
            </Text>
            <Show when={p().label}>
              <Text class={`shrink max-w-[45%] ml-1 py-0 px-[0.357rem] text-[0.857rem] font-bold whitespace-nowrap overflow-hidden bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`}>
                {p().label}
              </Text>
            </Show>
            <Show when={p().quantity > 0}>
              <Text class={`shrink ml-1 text-[1rem] font-bold tabular-nums whitespace-nowrap text-fg-muted text-shadow-xs text-shadow-bg/55 group-data-legendary/loot:text-gold compact:text-[0.929rem]`}>
                {p().currency ? t("hud.loot-entry.gain", { quantity: formatNumber(p().quantity) }) : t("hud.loot-entry.count", { quantity: formatNumber(p().quantity) })}
              </Text>
            </Show>
          </View>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLootFeedProps>> = {
  default: {"items":[{"id":"blade","name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false},{"id":"shard","name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true}]},
  rare: {"items":[{"id":"ring","name":"Tidewatch Ring","icon":0,"glyph":"gem","rarity":"rare","label":"Rare","quantity":0,"currency":false,"fading":false}]},
  empty: {"items":[]},
  overflow: {"items":[{"id":"long","name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false}]},
};
