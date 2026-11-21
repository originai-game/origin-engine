import { For, Icon, Show, Text, View, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./economy.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] font-bold leading-[1.5] py-0 px-1`;

export interface RpgLootProps {
  /** The source's name; empty uses Loot. */
  title?: string;
  /** Keyed { id, name, icon, count, description, available, reason, rarity }. rarity is common, uncommon, rare, epic, legendary or mythic (empty is common): the row's edge, the icon's socket and the name take its colour. The game supplies legality/capacity and removes collected rows after take or takeAll. Default []. */
  items?: ReadonlyArray<{ id: string; name: string; icon: string; count: number; description: string; available: boolean; reason: string; rarity: string }>;
  /** The game permits taking the displayed items together. Empty/loading/busy never sends takeAll. Default false. */
  canTakeAll?: boolean;
  /** Rewards are loading; existing items remain visible but cannot be collected. Default false. */
  loading?: boolean;
  /** A collection request is pending; disables take and takeAll to prevent duplicate requests. Default false. */
  busy?: boolean;
  /** The game's collection failure, such as a full inventory; empty hides it. */
  error?: string;
  /** The close event, with the row's key where it comes from a row. */
  onClose?: (e: UiEvent, key?: string) => void;
  /** The take event, with the row's key where it comes from a row. */
  onTake?: (e: UiEvent, key?: string) => void;
  /** The takeAll event, with the row's key where it comes from a row. */
  onTakeAll?: (e: UiEvent, key?: string) => void;
}

/**
 * What a chest, a body or a reward holds: a band with the source's name and
 * how many things it holds, one row per item (its icon in a socket and its
 * rarity on the row's edge, the name in that colour over what it is and,
 * when the game refuses it, why in words; the count on the name's line and
 * Take), and Close beside Take all. The game decides what can be taken; the
 * panel reports take (an item's key), takeAll and close (Close, or Escape
 * and pad B anywhere in it). The rows scroll inside the height given.
 */
export function RpgLoot(props: RpgLootProps) {
  return (
    <View class={`grow shrink basis-0 border-rim rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-glass-hi via-glass via-[96px] to-glass-lo shadow-panel text-fg p-4 gap-3 border disabled:text-fg-subtle`} label={props.title || t("rpg.loot.title")} busy={(props.loading ?? false) || (props.busy ?? false)} onAction={{ back: (e) => props.onClose?.(e) }}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-0.5`}>
        <Text class={`text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap`}>
          {(props.items ?? []).length ? t("rpg.loot.kicker", { count: formatNumber((props.items ?? []).length) }) : t("rpg.loot.title")}
        </Text>
        <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title || t("rpg.loot.title")}
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
      <Show when={(props.items ?? []).length} fallback={
          <Show when={!(props.loading ?? false)}>
            <Text class={`text-caption text-fg-muted text-center p-4 m-0`} role="status">
              {t("rpg.loot.empty")}
            </Text>
          </Show>
      }>
        <View class={`grow shrink basis-0 p-0 m-0 gap-1 overflow-auto`}>
          <For each={(props.items ?? [])}>
            {(item) => (
            <View class={`group/row rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-[#090e1a8c] text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#343d4f] grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-2 gap-y-1 pr-2 pl-3 border-l-3 border-border-strong py-1 hover:bg-[#151e32b2] hover:bg-none data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} data-not-available={!(item().available)} data-tier-common={(item().rarity || 'common') === "common"}>
              <View class={`justify-center items-center self-start bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none gap-2 size-[3.429rem] group-data-not-available/row:opacity-45 compact:row-span-2`}>
                <Icon class={`size-[78%]`} name={item().icon} />
              </View>
              <View class={`grow shrink basis-0 gap-0.5`}>
                <View class="flex-row items-baseline gap-2">
                  <Text class={`grow shrink basis-0 text-border-strong line-clamp-2 m-0 overflow-hidden group-data-not-available/row:opacity-45`} label={item().name}>
                    {item().name}
                  </Text>
                  <Text class={`tabular-nums text-fg whitespace-nowrap font-bold group-data-not-available/row:opacity-45`}>
                    {t("rpg.loot.count", { count: formatNumber(item().count) })}
                  </Text>
                </View>
                <Show when={!item().available} fallback={
                    <Show when={item().description}>
                      <Text class={`text-caption text-fg-muted m-0 compact:line-clamp-2 compact:overflow-hidden`}>
                        {item().description}
                      </Text>
                    </Show>
                }>
                  <Text class={`text-danger m-0`}>
                    {item().reason || t("rpg.economy.unavailable")}
                  </Text>
                </Show>
              </View>
              <View class={`min-h-11 compact:col-[2_/_4] compact:justify-self-end`} role="button" disabled={!item().available || (props.loading ?? false) || (props.busy ?? false)} label={t("rpg.loot.takeItem", { name: item().name })} onPress={(e) => props.onTake?.(e, item().id)}>
                <Text>{t("rpg.loot.take")}</Text>
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
      <View class={`flex-row items-center w-full mt-auto gap-2`}>
        <View class={`grow shrink basis-0 min-h-11 gap-1`} role="button" label={t("rpg.economy.close")} onPress={(e) => props.onClose?.(e)}>
          <Text glyph={"back"} class={keyCap} />
          <Text>{t("rpg.economy.close")}</Text>
        </View>
        <View class={`text-accent grow-1.4 shrink basis-0 min-h-11 gap-1`} role="button" disabled={!(props.items ?? []).length || !(props.canTakeAll ?? false) || (props.loading ?? false) || (props.busy ?? false)} onPress={(e) => props.onTakeAll?.(e)}>
          <Text>{t("rpg.loot.all")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLootProps>> = {
  default: {"title":"Abbey chest","items":[{"id":"ember","name":"Heart of the Ember","icon":"fire","count":1,"description":"Crafting reagent","available":true,"reason":"","rarity":"legendary"},{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Recovered from the chest.","available":true,"reason":"","rarity":"common"},{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full","rarity":"epic"}],"canTakeAll":true,"loading":false,"busy":false,"error":""},
  empty: {},
  loading: {"loading":true},
  busy: {"busy":true,"items":[{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Collecting…","available":true,"reason":"","rarity":"common"}],"canTakeAll":true},
  error: {"error":"Make room in your inventory and try again.","items":[{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full","rarity":"epic"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Close"} as const;
