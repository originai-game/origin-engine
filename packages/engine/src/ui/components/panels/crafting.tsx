import { For, Icon, Show, Text, View, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./economy.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] font-bold leading-[1.5] py-0 px-1`;

export interface RpgCraftingProps {
  /** The station name; empty uses Crafting. */
  title?: string;
  /** Keyed { id, name, icon, description, canCraft, reason, rarity, held } recipes. Uncraftable recipes remain selectable for inspection; reason explains the restriction. rarity (common … mythic, empty common) colours the row's edge, the socket and the name; held, how many of the result the player has as the game words it, is shown under the result (empty hides it). Default []. */
  recipes?: ReadonlyArray<{ id: string; name: string; icon: string; description: string; canCraft: boolean; reason: string; rarity: string; held: string }>;
  /** The selected recipe id, or empty for selection guidance. Clear the selection when removing its recipe. */
  selected?: string;
  /** Requirements for the selected recipe and quantity: keyed { id, name, quantity, sufficient }. The game formats quantity (2 / 3) and computes sufficiency. Default []. */
  ingredients?: ReadonlyArray<{ id: string; name: string; quantity: string; sufficient: boolean }>;
  /** The game-owned batch size. Increase/decrease intents carry the selected recipe id; the game validates and updates it. Default 1. */
  quantity?: number;
  /** The game permits a larger batch. Default false. */
  canIncrease?: boolean;
  /** The game permits a smaller batch. Default false. */
  canDecrease?: boolean;
  /** The complete batch cost as the game words it; empty hides it. */
  cost?: string;
  /** Recipe data is loading; existing recipes remain visible but inactive. Default false. */
  loading?: boolean;
  /** A craft is in progress; selection, quantities and crafting are inactive. Default false. */
  busy?: boolean;
  /** The game's failure message; empty hides it without discarding the selection. */
  error?: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The craft event, with the row's key where it comes from a row. */
  onCraft?: (e: UiEvent, key?: string) => void;
  /** The decrease event, with the row's key where it comes from a row. */
  onDecrease?: (e: UiEvent, key?: string) => void;
  /** The increase event, with the row's key where it comes from a row. */
  onIncrease?: (e: UiEvent, key?: string) => void;
}

/**
 * A crafting station: a band with the station's name, the recipes as rows
 * (the result's icon in a socket, its rarity on the row's edge and in its
 * name, why it cannot be made in words), and beside them (under them when
 * narrow) the selected recipe: its art large over a glow of its rarity,
 * what it does and how many the player holds, the ingredients as have /
 * need with a short one marked in words, the batch stepper with the cost,
 * and Craft. The game decides every requirement; the panel reports choose,
 * increase, decrease and craft, each with the recipe's key.
 */
export function RpgCrafting(props: RpgCraftingProps) {
  return (
    <View class={`grow shrink basis-0 border-rim rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-glass-hi via-glass via-[96px] to-glass-lo shadow-panel text-fg w-full p-4 gap-3 border disabled:text-fg-subtle`} label={props.title || t("rpg.craft.title")} busy={(props.loading ?? false) || (props.busy ?? false)}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-0.5`}>
        <Text class={`text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap`}>
          {t("rpg.craft.title")}
        </Text>
        <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title || t("rpg.craft.title")}
        </Text>
      </View>
      <Show when={props.error}>
        <Text class={`text-danger bg-danger/12 shadow-[inset_3px_0px_0px_#ff9599] py-1 px-2 m-0`} role="alert">
          {props.error}
        </Text>
      </Show>
      <Show when={(props.loading ?? false) || (props.busy ?? false)}>
        <View class={`flex-row items-center bg-tile shadow-sunk p-2 gap-2 rounded-control`} role="status">
          <Spinner name={(props.busy ?? false) ? t("rpg.economy.busy") : t("rpg.economy.loading")} />
          <Text class="shrink">{(props.busy ?? false) ? t("rpg.economy.busy") : t("rpg.economy.loading")}</Text>
        </View>
      </Show>
      <Show when={(props.recipes ?? []).length} fallback={
          <Show when={!(props.loading ?? false)}>
            <Text class={`text-caption text-fg-muted text-center p-4 m-0`} role="status">
              {t("rpg.craft.empty")}
            </Text>
          </Show>
      }>
        <View class="grow shrink basis-0 grid grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)] gap-x-4 gap-y-3 w-full compact:flex compact:flex-col">
          <View class={`gap-1 overflow-auto compact:grow-0 compact:shrink compact:basis-auto compact:max-h-2/5`} role="listbox" label={t("rpg.craft.title")}>
            <For each={(props.recipes ?? [])}>
              {(recipe) => (
              <View class={`group/row rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-[#090e1a8c] text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#343d4f] w-full min-h-11 pr-2 pl-3 border-t-0 border-r-0 border-b-0 border-l-3 border-border-strong py-1 hover:bg-[#151e32b2] hover:bg-none data-selected:bg-transparent data-selected:bg-linear-to-r data-selected:from-[#38e8ff3d] data-selected:via-[#38e8ff14] data-selected:via-55% data-selected:to-[#38e8ff08] data-selected:text-fg data-selected:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="option" selected={recipe().id === props.selected} disabled={(props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onChoose?.(e, recipe().id)} data-not-can={!(recipe().canCraft)} data-selected={recipe().id === props.selected} data-tier-common={(recipe().rarity || 'common') === "common"}>
                <View class={`items-center w-full gap-2`}>
                  <View class={`justify-center items-center bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none gap-2 size-[3.143rem] group-data-not-can/row:opacity-45`}>
                    <Icon class={`size-[74%]`} name={recipe().icon} />
                  </View>
                  <View class="items-start text-start gap-0.5 shrink">
                    <Text class={`text-border-strong font-semibold`}>
                      {recipe().name}
                    </Text>
                    <Show when={!recipe().canCraft}>
                      <Text class={`text-danger`}>
                        {recipe().reason || t("rpg.economy.unavailable")}
                      </Text>
                    </Show>
                  </View>
                </View>
              </View>
              )}
            </For>
          </View>
          <Show when={!props.selected}>
            <Text class={`text-caption text-fg-muted text-center p-4 m-0`}>
              {t("rpg.craft.choose")}
            </Text>
          </Show>
          <For each={(props.recipes ?? [])}>
            {(recipe) => (
            <Show when={recipe().id === props.selected}>
              <View class={`gap-0 compact:grow compact:shrink compact:basis-0`} label={recipe().name} data-tier-common={(recipe().rarity || 'common') === "common"}>
                <View class="shrink w-full gap-3 overflow-auto">
                  <View class={`items-center justify-center text-border-strong`}>
                    <View class="absolute opacity-35 inset-0" />
                    <Icon class="shrink" name={recipe().icon} />
                  </View>
                  <View class={`gap-0.5`}>
                    <Text class="text-[20.64px] text-border-strong m-0">{recipe().name}</Text>
                    <Show when={recipe().description}>
                      <Text class={`text-caption text-fg-muted m-0`}>
                        {recipe().description}
                      </Text>
                    </Show>
                    <Show when={recipe().held}>
                      <Text class={`tabular-nums text-fg font-bold m-0`}>
                        {recipe().held}
                      </Text>
                    </Show>
                  </View>
                  <Show when={(props.ingredients ?? []).length}>
                    <View class="gap-1">
                      <Text class={`text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap m-0`}>
                        {t("rpg.craft.ingredients")}
                      </Text>
                      <View class={`grid grid-cols-[repeat(auto-fill,minmax(calc(130_*_calc(1rem_/_14)),1fr))] p-0 m-0 gap-1`}>
                        <For each={(props.ingredients ?? [])}>
                          {(ingredient) => (
                          <View class={`group/ingredient bg-[#090e1a8c] shadow-[inset_0px_0px_0px_1px_#343d4f] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none py-1 px-2 gap-0.5 data-not-sufficient:bg-none data-not-sufficient:bg-danger/10 data-not-sufficient:shadow-[inset_0px_0px_0px_1px_#ff95998c]`} data-not-sufficient={!(ingredient().sufficient)}>
                            <Text class={`text-fg-muted`}>
                              {ingredient().name}
                            </Text>
                            <Text class={`tabular-nums text-fg text-[18.24px] font-bold whitespace-nowrap group-data-not-sufficient/ingredient:text-danger`}>
                              {ingredient().quantity}
                            </Text>
                            <Show when={!ingredient().sufficient}>
                              <Text class="text-danger">{t("rpg.craft.missing")}</Text>
                            </Show>
                          </View>
                          )}
                        </For>
                      </View>
                    </View>
                  </Show>
                  <Show when={!recipe().canCraft}>
                    <Text class={`text-danger m-0`}>
                      {recipe().reason || t("rpg.economy.unavailable")}
                    </Text>
                  </Show>
                </View>
                <View class="w-full pt-2 border-t border-rim-soft gap-1">
                  <View class="flex-row items-center justify-between flex-wrap gap-2">
                    <View class={`flex-row items-center gap-0`}>
                      <View class={`min-w-11 min-h-11 shrink`} role="button" label={t("rpg.economy.less")} disabled={!(props.canDecrease ?? false) || (props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onDecrease?.(e, recipe().id)}>
                        <Text>{t("rpg.economy.minus")}</Text>
                      </View>
                      <View class={`text-fg grid items-center justify-items-center min-w-[4.571rem] min-h-11 text-[18.24px] font-bold bg-[#04060c8c] shadow-[inset_0px_0px_0px_1px_#ffffff0f] shrink`} label={t("rpg.economy.quantity", { quantity: (props.quantity ?? 1) })}>
                        <Text class="tabular-nums">{formatNumber((props.quantity ?? 1))}</Text>
                      </View>
                      <View class={`min-w-11 min-h-11 shrink`} role="button" label={t("rpg.economy.more")} disabled={!(props.canIncrease ?? false) || (props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onIncrease?.(e, recipe().id)}>
                        <Text>{t("rpg.economy.plus")}</Text>
                      </View>
                    </View>
                    <Show when={props.cost}>
                      <Text class={`tabular-nums text-fg text-[18.24px] font-bold whitespace-nowrap m-0 shrink`}>
                        {props.cost}
                      </Text>
                    </Show>
                  </View>
                  <View class={`text-accent w-full min-h-[50.6px] gap-1`} role="button" disabled={!recipe().canCraft || (props.loading ?? false) || (props.busy ?? false)} label={t("rpg.craft.make")} onPress={(e) => props.onCraft?.(e, recipe().id)}>
                    <Text glyph={"details"} class={keyCap} />
                    <Text>{t("rpg.craft.make")}</Text>
                  </View>
                </View>
              </View>
            </Show>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgCraftingProps>> = {
  default: {"title":"Alchemist’s bench","recipes":[{"id":"draught","name":"Healing draught","icon":"potion","description":"Restores 240 health over 4 s. Produces 2 draughts.","canCraft":true,"reason":"","rarity":"uncommon","held":"You hold 5"},{"id":"elixir","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Requires Alchemy 4","rarity":"epic","held":""}],"selected":"draught","ingredients":[{"id":"herb","name":"Silverleaf","quantity":"6 / 2","sufficient":true},{"id":"water","name":"Pure water","quantity":"4 / 1","sufficient":true}],"quantity":1,"canIncrease":true,"canDecrease":false,"cost":"10 gold","loading":false,"busy":false,"error":""},
  empty: {},
  loading: {"loading":true},
  busy: {"busy":true,"selected":"p","recipes":[{"id":"p","name":"Healing draught","icon":"potion","description":"Brewing a batch.","canCraft":true,"reason":"","rarity":"uncommon","held":""}]},
  missing: {"selected":"e","recipes":[{"id":"e","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Missing ingredients","rarity":"legendary","held":""}],"ingredients":[{"id":"c","name":"Storm crystal","quantity":"0 / 1","sufficient":false}],"error":"Not enough ingredients."},
};
