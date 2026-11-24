import { For, Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface PickupItem {
  id: string;
  /** What the game calls it. */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** The count gained (+30). */
  quantity?: number;
  /** common, rare, epic or legendary: the edge's and the count's colour. */
  rarity?: string;
}

export interface ShooterPickupsProps {
  /**
   * What the player just picked up, newest first. The game trims old rows
   * (a phone shows the newest); a new row slides in from the screen's edge
   * and an old one fades out. Read only.
   */
  items?: ReadonlyArray<PickupItem>;
}

/**
 * The pickup feed lower on the screen's side: each row an icon, the item's
 * name and the count gained on a dark glass that fades toward the middle,
 * its edge at the screen's side tinted by rarity. Read only. A row the For
 * removes fades out on exit: by itself; the rows' FLIP reflow (the svelte's
 * animate:flip) has no TSX surface and is dropped, as is the row's
 * right-to-left mirror.
 */
export function ShooterPickups(props: ShooterPickupsProps) {
  return (
    <View class="flex-col items-end gap-y-1.25 max-w-full text-fg" role="log" label={t("shooter.pickups.name")}>
      <For each={props.items ?? []} key={(p) => p.id}>
        {(p, i) => (
          <View
            class={`group/pickup relative flex-row items-center min-w-0 max-w-full pl-2 bg-glass compact:data-later:hidden transition duration-base ease-out starting:opacity-0 starting:translate-x-4 exit:opacity-0 ${packClass("Plate", "frame")}`}
            label={t("shooter.pickups.row", { name: p().name, quantity: p().quantity ?? 0 })}
            data-rarity-rare={p().rarity === "rare"}
            data-rarity-epic={p().rarity === "epic"}
            data-rarity-legendary={p().rarity === "legendary"}
            data-later={i() >= 1}
          >
            <View class="shrink-0 my-0.5 size-5.5 compact:size-4.5 nine-chip-cut overflow-hidden bg-plate-hi">
              <Show when={p().icon} fallback={
                <Show when={p().glyph}>
                  <Icon class="size-full text-fg" name={p().glyph} />
                </Show>
              }>
                <Image class="size-full" src={p().icon} />
              </Show>
            </View>
            <Text class="min-w-0 text-sm font-semibold whitespace-nowrap truncate text-shadow-tag text-stroke-3 text-stroke-bg compact:text-caption">{p().name}</Text>
            <Text
              class={`shrink-0 text-sm font-bold tabular-nums whitespace-nowrap text-fg-muted text-shadow-tag text-stroke-3 text-stroke-bg compact:text-caption group-data-rarity-rare/pickup:text-rare group-data-rarity-epic/pickup:text-epic group-data-rarity-legendary/pickup:text-gold`}
            >
              {t("shooter.pickups.count", { quantity: formatNumber(p().quantity ?? 0) })}
            </Text>
            <View
              class={`shrink-0 self-stretch w-0.75 ml-1 bg-fg group-data-rarity-rare/pickup:bg-rare group-data-rarity-epic/pickup:bg-epic group-data-rarity-legendary/pickup:bg-gold`}
            />
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterPickupsProps>> = {
  default: {"items":[{"id":"frag","name":"Frag grenade","icon":0,"glyph":"bomb","quantity":1,"rarity":"legendary"},{"id":"cell","name":"Shield cell","icon":0,"glyph":"shield","quantity":2,"rarity":"rare"},{"id":"light","name":"Light rounds","icon":0,"glyph":"bolt","quantity":30,"rarity":"common"}]},
  epic: {"items":[{"id":"optic","name":"Holo optic","icon":0,"glyph":"gem","quantity":1,"rarity":"epic"}]},
  overflow: {"items":[{"id":"long","name":"Extended heavy magazine for the Northern Relay marksman rifle","icon":0,"glyph":"","quantity":123456,"rarity":"legendary"}]},
};
