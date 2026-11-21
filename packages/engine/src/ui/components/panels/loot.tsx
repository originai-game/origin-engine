import { For, Icon, Show, Text, View, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { Spinner } from "../base/spinner";
import "./economy.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden text-xl leading-none`;

/** An item's socket: its ground tinted by its rarity, the icon over it. */
const socket =
  "relative flex-row items-center justify-center nine-plate-cut bg-tile group-data-tier-uncommon/row:text-uncommon group-data-tier-rare/row:text-rare group-data-tier-epic/row:text-epic group-data-tier-legendary/row:text-legendary group-data-tier-mythic/row:text-mythic";

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
 *
 * Dropped against the .svelte: the socket's rarity rim (the rarity tints
 * the socket's ground instead; a box shadow cannot follow the cut shape),
 * the art's drop-shadow filter, and the focused row's wash gradient and
 * inset rim (a flat wash and the leading edge stand in; a cut element
 * takes one bg colour and no box shadow).
 */
export function RpgLoot(props: RpgLootProps) {
  const items = () => props.items ?? [];
  const loading = () => props.loading ?? false;
  const busy = () => props.busy ?? false;
  return (
    <View
      class={`relative grow shrink basis-0 min-h-0 nine-panel-cut bg-glass text-fg p-4 gap-3 disabled:text-fg-subtle ${packClass("Panel", "frame")}`}
      label={props.title || t("rpg.loot.title")}
      busy={loading() || busy()}
      onAction={{ back: (e) => props.onClose?.(e) }}
    >
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class="shrink border-b border-rim-soft w-full py-1 px-4 gap-0.5">
        <Text class="text-xs font-semibold tracking-kicker uppercase text-fg-subtle whitespace-nowrap">
          {items().length ? t("rpg.loot.kicker", { count: formatNumber(items().length) }) : t("rpg.loot.title")}
        </Text>
        <Text class="font-display text-base font-bold leading-tight tracking-tight text-fg m-0">
          {props.title || t("rpg.loot.title")}
        </Text>
      </View>
      <Show when={props.error}>
        <Text class="text-danger inset-shadow-bar inset-shadow-danger py-1 px-2 m-0" role="alert">
          {props.error}
        </Text>
      </Show>
      <Show when={loading() || busy()}>
        <View class="flex-row items-center bg-tile shadow-sunk p-2 gap-2 rounded-control" role="status">
          <Spinner name={busy() ? t("rpg.economy.busy") : t("rpg.economy.loading")} />
          <Text class="shrink">{busy() ? t("rpg.economy.busy") : t("rpg.economy.loading")}</Text>
        </View>
      </Show>
      <Show when={items().length} fallback={
        <Show when={!loading()}>
          <Text class="text-caption text-fg-muted text-center p-4 m-0" role="status">
            {t("rpg.loot.empty")}
          </Text>
        </Show>
      }>
        <View class="grow shrink basis-0 min-h-0 p-0 m-0 gap-1 overflow-auto">
          <For each={items()}>
            {(item) => (
              <View
                class={`group/row grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-2 gap-y-1 pr-2 pl-3 py-1 nine-plate-cut border-l-3 border-transparent hover:bg-surface-hover focus-within:border-hud-accent transition duration-317 ${packClass("Row", "frame")} group-data-tier-uncommon/row:border-uncommon group-data-tier-rare/row:border-rare group-data-tier-epic/row:border-epic group-data-tier-legendary/row:border-legendary group-data-tier-mythic/row:border-mythic`}
                data-not-available={!item().available}
                data-tier-common={(item().rarity || "common") === "common"}
                data-tier-uncommon={(item().rarity || "common") === "uncommon"}
                data-tier-rare={(item().rarity || "common") === "rare"}
                data-tier-epic={(item().rarity || "common") === "epic"}
                data-tier-legendary={(item().rarity || "common") === "legendary"}
                data-tier-mythic={(item().rarity || "common") === "mythic"}
              >
                <View class={`${socket} self-start gap-2 size-12 group-data-not-available/row:opacity-45 compact:row-span-2`}>
                  <Show when={(item().rarity || "common") !== "common"}>
                    <View class="absolute inset-0 nine-plate-cut bg-current opacity-30" />
                  </Show>
                  <Icon class="relative size-39/50" name={item().icon} />
                </View>
                <View class="grow shrink basis-0 min-w-0 gap-0.5">
                  <View class="flex-row items-baseline gap-2 min-w-0">
                    <Text class={`grow shrink basis-0 min-w-0 text-fg group-data-tier-uncommon/row:text-uncommon group-data-tier-rare/row:text-rare group-data-tier-epic/row:text-epic group-data-tier-legendary/row:text-legendary group-data-tier-mythic/row:text-mythic line-clamp-2 m-0 overflow-hidden group-data-not-available/row:opacity-45`} label={item().name}>
                      {item().name}
                    </Text>
                    <Text class="tabular-nums text-fg whitespace-nowrap font-bold group-data-not-available/row:opacity-45">
                      {t("rpg.loot.count", { count: formatNumber(item().count) })}
                    </Text>
                  </View>
                  <Show when={!item().available} fallback={
                    <Show when={item().description}>
                      <Text class="text-caption text-fg-muted m-0 compact:line-clamp-2 compact:overflow-hidden">
                        {item().description}
                      </Text>
                    </Show>
                  }>
                    <Text class="text-danger m-0">
                      {item().reason || t("rpg.economy.unavailable")}
                    </Text>
                  </Show>
                </View>
                <Button
                  class={`shrink-0 min-h-11 compact:col-[2_/_4] compact:justify-self-end ${packClass("Button", "plate")}`}
                  disabled={!item().available || loading() || busy()}
                  label={t("rpg.loot.takeItem", { name: item().name })}
                  onPress={(e) => props.onTake?.(e, item().id)}
                >
                  <Text>{t("rpg.loot.take")}</Text>
                </Button>
              </View>
            )}
          </For>
        </View>
      </Show>
      <View class="shrink flex-row items-center w-full mt-auto gap-2">
        <Button class={`grow shrink basis-0 min-h-11 gap-1 ${packClass("Button", "plate")}`} label={t("rpg.economy.close")} onPress={(e) => props.onClose?.(e)}>
          <Text glyph="back" class={keyCap} />
          <Text>{t("rpg.economy.close")}</Text>
        </Button>
        <Button variant="primary" class={`grow-[1.4] shrink basis-0 min-h-11 gap-1 ${packClass("Button", "plate", "primary")}`} disabled={!items().length || !props.canTakeAll || loading() || busy()} onPress={(e) => props.onTakeAll?.(e)}>
          <Text>{t("rpg.loot.all")}</Text>
        </Button>
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
