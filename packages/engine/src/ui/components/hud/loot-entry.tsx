import { Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./loot-entry.en.ogt";

export interface HudLootEntryProps {
  /** The item's name (Ember Blade). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** common, uncommon, rare, epic or legendary: the edge's colour (on data-rarity); an epic or legendary item's name is inked too. */
  rarity?: string;
  /** The rarity as the game words it (Epic), on a tag; empty is none. */
  label?: string;
  /** The count gained, written ×3 (or +1,240 when currency is true); 0 hides it. */
  quantity?: number;
  /** The quantity is money (gold): written as a gain. */
  currency?: boolean;
  /** An old row on its way out: dimmed. */
  fading?: boolean;
}

/**
 * One row of a HUD's loot feed, a block a feed is built from: an icon,
 * the item's name, its rarity on a tag and the count gained, on a dark
 * glass that fades toward the middle with its edge at the screen's side
 * tinted by rarity; a fading row dims. Read only. The art's cut corner
 * (corner-shape) has no TSX surface: a rounded one stands in.
 */
export function HudLootEntry(props: HudLootEntryProps) {
  const rarity = () => props.rarity ?? "common";
  return (
    <View
      class={`flex-row items-center w-fit min-w-0 max-w-full pr-3 bg-linear-to-r from-[#1a1e28db] to-[#1a1e2859] data-fading:opacity-55`}
      role="group"
      label={(props.quantity ?? 0) > 0 ? t("hud.loot-entry.row", { name: props.name, quantity: props.currency ? t("hud.loot-entry.gain", { quantity: formatNumber(props.quantity ?? 0) }) : t("hud.loot-entry.count", { quantity: formatNumber(props.quantity ?? 0) }) }) : props.name}
      data-rarity={rarity()}
      data-fading={props.fading}
    >
      <View class={`shrink-0 self-stretch w-[0.214rem] mr-1 bg-white/60 data-uncommon:bg-[#55e39a] data-rare:bg-[#5cc8ff] data-epic:bg-[#c86ff5] data-legendary:bg-[#ffcf5a]`} data-uncommon={rarity() === "uncommon"} data-rare={rarity() === "rare"} data-epic={rarity() === "epic"} data-legendary={rarity() === "legendary"} />
      <View class="relative shrink-0 my-[0.214rem] size-[1.429rem] nine-chip-cut overflow-hidden bg-[#2a2d38] compact:size-[1.286rem]">
        <Show when={props.icon} fallback={
          <Show when={props.glyph}>
            <Icon class="absolute inset-0 size-full text-fg" name={props.glyph} />
          </Show>
        }>
          <Image class="absolute inset-0 size-full" src={props.icon} />
        </Show>
      </View>
      <Text class={`min-w-0 text-[1rem] font-semibold whitespace-nowrap truncate text-fg text-shadow-2xs data-epic:text-[#dcb8ff] data-legendary:text-[#ffcf5a]`} data-epic={rarity() === "epic"} data-legendary={rarity() === "legendary"}>
        {props.name}
      </Text>
      <Show when={props.label}>
        <Text class={`shrink min-w-0 py-0 px-[0.357rem] text-[0.857rem] font-bold whitespace-nowrap truncate text-fg nine-chip-cut bg-[#1b1f29e0] ${packClass("Chip", "frame")}`}>
          {props.label}
        </Text>
      </Show>
      <Show when={(props.quantity ?? 0) > 0}>
        <Text class={`shrink-0 text-[1rem] font-bold tabular-nums whitespace-nowrap text-white/66 text-shadow-2xs data-legendary:text-[#ffcf5a]`} data-legendary={rarity() === "legendary"}>
          {props.currency ? t("hud.loot-entry.gain", { quantity: formatNumber(props.quantity ?? 0) }) : t("hud.loot-entry.count", { quantity: formatNumber(props.quantity ?? 0) })}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudLootEntryProps>> = {
  default: {"name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false},
  gold: {"name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false},
  fading: {"name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true},
  empty: {"name":"","icon":0,"glyph":"","rarity":"common","label":"","quantity":0,"currency":false,"fading":false},
  overflow: {"name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false},
};
