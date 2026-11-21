import { For, Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./economy.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] font-bold leading-[1.5] py-0 px-1`;

export interface RpgTradeProps {
  /** The merchant or trade title; empty uses Trade. */
  title?: string;
  /** buy or sell. The game changes this in response to switchMode and supplies that side's goods and prices. Default "buy". */
  mode?: string;
  /** The player's balance as the game words it; empty hides it. */
  wallet?: string;
  /** Keyed { id, name, icon, stock, description, price, was, after, available, reason, rarity }. stock, price (the total), was (the struck price before a discount, empty none) and after (the balance once paid, empty hides it) are game-formatted strings; available includes balance, capacity, quantity and legality checks; rarity (common … mythic, empty common) colours the row's edge, the socket and the name. Default []. */
  goods?: ReadonlyArray<{ id: string; name: string; icon: string; stock: string; description: string; price: string; was: string; after: string; available: boolean; reason: string; rarity: string }>;
  /** The inspected good's id. Unavailable goods remain selectable so their restriction can be read. */
  selected?: string;
  /** Goods are loading; selection and transactions are disabled. Default false. */
  loading?: boolean;
  /** A transaction is pending; selection, mode switching and transactions are disabled. Default false. */
  busy?: boolean;
  /** The game's transaction failure, kept visible alongside the selection. */
  error?: string;
  /** The buy event, with the row's key where it comes from a row. */
  onBuy?: (e: UiEvent, key?: string) => void;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The sell event, with the row's key where it comes from a row. */
  onSell?: (e: UiEvent, key?: string) => void;
  /** The switchMode event, with the row's key where it comes from a row. */
  onSwitchMode?: (e: UiEvent, key?: string) => void;
}

/**
 * A merchant: a band with the merchant's name, the Buy / Sell switch and
 * the purse; an alert when a deal failed; the goods as rows (the item's
 * icon in a socket, its rarity on the row's edge and in its name, its stock
 * or why it cannot be had, and its price, a discount struck before it);
 * and at the foot the selected good with its total, the balance after, and
 * Buy or Sell. The game decides every price and verdict; the panel reports
 * choose, switchMode, buy and sell, each with the good's key.
 */
export function RpgTrade(props: RpgTradeProps) {
  return (
    <View class={`grow shrink basis-0 border-rim rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-glass-hi via-glass via-[96px] to-glass-lo shadow-panel text-fg w-full p-4 gap-3 border disabled:text-fg-subtle`} label={props.title || t("rpg.trade.title")} busy={(props.loading ?? false) || (props.busy ?? false)}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <View class="grow shrink basis-0 gap-0.5 compact:basis-full">
          <Text class={`text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap`}>
            {t("rpg.trade.kicker")}
          </Text>
          <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
            {props.title || t("rpg.trade.title")}
          </Text>
        </View>
        <View class={`min-h-11 bg-[#04060c8c] shadow-[inset_0px_0px_0px_1px_#4e586c] p-0.5 gap-0.5 rounded-control`} role="switch" checked={(props.mode ?? "buy") === 'sell'} label={(props.mode ?? "buy") === 'sell' ? t("rpg.trade.toBuy") : t("rpg.trade.toSell")} disabled={(props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onSwitchMode?.(e)}>
          <Text class="text-fg-muted py-1 px-2 rounded-control data-on:text-fg data-on:bg-none data-on:bg-material-edge data-on:shadow-[inset_0px_1px_0px_#ffffff33]" data-on={(props.mode ?? "buy") !== 'sell'}>
            {t("rpg.trade.buy")}
          </Text>
          <Text class="text-fg-muted py-1 px-2 rounded-control data-on:text-fg data-on:bg-none data-on:bg-material-edge data-on:shadow-[inset_0px_1px_0px_#ffffff33]" data-on={(props.mode ?? "buy") === 'sell'}>
            {t("rpg.trade.sell")}
          </Text>
        </View>
        <Show when={props.wallet}>
          <View class={`flex-row items-center text-fg pr-2 pl-0.5 bg-[#04060c8c] shadow-[inset_0px_0px_0px_1px_#4e586c] font-bold py-0.5 gap-1 rounded-capsule`}>
            <Text class="whitespace-nowrap tabular-nums size-[1.571rem] rounded-capsule" />
            <Text class="whitespace-nowrap tabular-nums shrink">{props.wallet}</Text>
          </View>
        </Show>
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
      <Show when={(props.goods ?? []).length} fallback={
          <Show when={!(props.loading ?? false)}>
            <Text class={`text-caption text-fg-muted text-center p-4 m-0`} role="status">
              {t("rpg.trade.empty")}
            </Text>
          </Show>
      }>
        <View class={`grow shrink basis-0 gap-1 overflow-auto`} role="listbox" label={(props.mode ?? "buy") === 'sell' ? t("rpg.trade.sell") : t("rpg.trade.buy")}>
          <For each={(props.goods ?? [])}>
            {(good) => (
            <View class={`group/row rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-[#090e1a8c] text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#343d4f] w-full min-h-11 pr-2 pl-3 border-t-0 border-r-0 border-b-0 border-l-3 border-border-strong py-1 hover:bg-[#151e32b2] hover:bg-none data-selected:bg-transparent data-selected:bg-linear-to-r data-selected:from-[#38e8ff3d] data-selected:via-[#38e8ff14] data-selected:via-55% data-selected:to-[#38e8ff08] data-selected:text-fg data-selected:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="option" selected={good().id === props.selected} disabled={(props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onChoose?.(e, good().id)} data-not-available={!(good().available)} data-selected={good().id === props.selected} data-tier-common={(good().rarity || 'common') === "common"}>
              <View class={`grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-2 gap-y-0.5 w-full`}>
                <View class={`justify-center items-center bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none gap-2 size-[3.143rem] group-data-not-available/row:opacity-45 compact:row-span-2`}>
                  <Icon class={`size-[74%]`} name={good().icon} />
                </View>
                <View class="grow shrink basis-0 items-start text-start gap-0.5">
                  <Text class={`text-border-strong font-semibold`}>
                    {good().name}
                  </Text>
                  <Show when={!good().available} fallback={
                      <Text class={`text-caption text-fg-muted`}>
                        {good().stock}
                      </Text>
                  }>
                    <Text class={`text-danger`}>
                      {good().reason || t("rpg.economy.unavailable")}
                    </Text>
                  </Show>
                </View>
                <View class={`text-fg items-baseline font-bold gap-1 group-data-not-available/row:text-danger compact:col-[2] compact:justify-self-start`}>
                  <Show when={good().was}>
                    <Text class={`line-through text-fg-muted font-semibold whitespace-nowrap tabular-nums shrink`} label={t("rpg.trade.was", { price: good().was })}>
                      {good().was}
                    </Text>
                  </Show>
                  <Text class="whitespace-nowrap tabular-nums shrink">{good().price}</Text>
                </View>
              </View>
            </View>
            )}
          </For>
        </View>
        <Show when={!props.selected}>
          <Text class={`text-caption text-fg-muted text-center p-4 m-0`}>
            {t("rpg.trade.choose")}
          </Text>
        </Show>
        <For each={(props.goods ?? [])}>
          {(good) => (
          <Show when={good().id === props.selected}>
            <View class={`w-full pt-2 border-t border-rim-soft gap-2`} label={good().name} data-tier-common={(good().rarity || 'common') === "common"}>
              <View class="items-center w-full gap-2 compact:flex-wrap">
                <View class="justify-center items-center bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none gap-2 size-[4.286rem]">
                  <Icon class="size-[74%]" name={good().icon} />
                </View>
                <View class="grow shrink basis-0 items-start text-start gap-0.5">
                  <Text class="font-semibold text-border-strong">{good().name}</Text>
                  <Show when={!good().available} fallback={
                      <Show when={good().description}>
                        <Text class={`text-caption text-fg-muted`}>
                          {good().description}
                        </Text>
                      </Show>
                  }>
                    <Text class="text-danger">{good().reason || t("rpg.economy.unavailable")}</Text>
                  </Show>
                </View>
                <View class="items-end text-end gap-0.5 compact:flex-row compact:basis-full compact:items-baseline compact:justify-end compact:gap-2">
                  <Text class="text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap">
                    {t("rpg.trade.total")}
                  </Text>
                  <Text class={`tabular-nums text-fg text-[20.64px] font-bold whitespace-nowrap`} label={t("rpg.trade.price", { price: good().price })}>
                    {good().price}
                  </Text>
                  <Show when={good().after}>
                    <Text class={`text-caption text-fg-muted whitespace-nowrap`}>
                      {good().after}
                    </Text>
                  </Show>
                </View>
              </View>
              <Show when={(props.mode ?? "buy") === 'sell'} fallback={
                  <View class={`text-accent w-full min-h-[50.6px] gap-1`} role="button" disabled={!good().available || (props.loading ?? false) || (props.busy ?? false)} label={t("rpg.trade.buy")} onPress={(e) => props.onBuy?.(e, good().id)}>
                    <Text glyph={"details"} class={keyCap} />
                    <Text>{t("rpg.trade.buy")}</Text>
                  </View>
              }>
                <View class={`text-accent w-full min-h-[50.6px] gap-1`} role="button" disabled={!good().available || (props.loading ?? false) || (props.busy ?? false)} label={t("rpg.trade.sell")} onPress={(e) => props.onSell?.(e, good().id)}>
                  <Text glyph={"details"} class={keyCap} />
                  <Text>{t("rpg.trade.sell")}</Text>
                </View>
              </Show>
            </View>
          </Show>
          )}
        </For>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgTradeProps>> = {
  default: {"title":"Mira’s supplies","wallet":"120 gold","mode":"buy","goods":[{"id":"potion","name":"Healing draught ×2","icon":"potion","stock":"8 in stock","description":"Two draughts for the road.","price":"20 gold","was":"","after":"120 → 100","available":true,"reason":"","rarity":"uncommon"},{"id":"lamp","name":"Hooded lantern","icon":"fire","stock":"2 in stock","description":"Burns through the night.","price":"45 gold","was":"60 gold","after":"120 → 75","available":true,"reason":"","rarity":"rare"},{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","was":"","after":"","available":false,"reason":"Not enough gold","rarity":"epic"}],"selected":"potion","loading":false,"busy":false,"error":""},
  empty: {},
  loading: {"loading":true},
  sell: {"mode":"sell","wallet":"120 gold","selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"3 owned","description":"Sell one crystal.","price":"80 gold","was":"","after":"120 → 200","available":true,"reason":"","rarity":"epic"}]},
  unavailable: {"selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","was":"","after":"","available":false,"reason":"Not enough gold","rarity":"epic"}],"error":"The price changed. Review it before trying again."},
};
