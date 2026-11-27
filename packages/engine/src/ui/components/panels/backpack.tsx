import { For, Icon, Scroll, Show, Text, View, formatCompact, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Tab, Tabs } from "../../controls/tabs";
import "./backpack.en.strings.ts";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden text-lg leading-none`;

export interface RpgBackpackProps {
  /** The bag's heading (Backpack, Stash, a chest's name). */
  title: string;
  /** A small line over the title naming the screen (Inventory); empty hides it. */
  kicker: string;
  /** The categories, a keyed collection of { id, label, count }: count, how many items the category holds, is shown beside its label above 0. */
  tabs: ReadonlyArray<{ id: string; label: string; count: number }>;
  /** The id of the category shown. */
  tab: string;
  /** The items shown, in the game's order: each { id, name, icon, count, rarity, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; count above 1 is shown; fresh marks an unseen item, locked one the game will not let go, broken one it will not equip. */
  items: ReadonlyArray<{ id: string; name: string; icon: string; count: number; rarity: string; equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }>;
  /** The selected item's id; empty is none. */
  selected: string;
  /** How full the bag is, as the game words it (30 / 36 slots); empty hides it. */
  capacity: string;
  /** The load as the game words it (92.4 / 120); empty hides it. */
  weight: string;
  /** The load from 0 to 1 for its bar; above 1 is overburdened. */
  load: number;
  /** The purse as the game formats it; empty hides it. */
  gold: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The choosetab event, with the row's key where it comes from a row. */
  onChoosetab?: (e: UiEvent, key?: string) => void;
  /** The tabNext event, with the row's key where it comes from a row. */
  onTabNext?: (e: UiEvent, key?: string) => void;
  /** The tabPrev event, with the row's key where it comes from a row. */
  onTabPrev?: (e: UiEvent, key?: string) => void;
  /** A tile dropped on another: the dragged item's key and the target's. A locked tile is not picked up; the dragged tile goes home, and the game reorders, swaps or stacks as it sees fit. */
  onMove?: (e: UiEvent, move: { dragged: string; target: string }) => void;
}

/**
 * A bag of items: a band with a kicker, the title and the purse, category
 * tabs between the action's key caps, how full the bag is, a grid of item
 * tiles and a footer with the carried weight. The game owns what a
 * category holds, the order and every number; the bag reports choose (an
 * item's key), choosetab (a category's key), tabNext and tabPrev. A tile
 * drags onto another (a locked one stays put) and the bag reports move
 * with { dragged, target }, the two items' keys. A tile
 * shows its rarity as its bar along the foot and its tags; its stack,
 * Equipped, New, Locked and Broken sit in its corners, each named in
 * words for a screen reader, Broken also on the tile. The selected tile
 * wears corner brackets. The grid fills the height it is given and
 * scrolls inside it.
 */
export function RpgBackpack(props: RpgBackpackProps) {
  return (
    <View class={`grow shrink basis-0 p-4 gap-2 disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-border-muted w-full py-1 px-4 gap-3 compact:py-1`}>
        <View class="grow shrink basis-30 gap-0.5">
          <Show when={props.kicker}>
            <Text class={`text-xs font-semibold tracking-kicker uppercase text-fg-muted whitespace-nowrap text-ellipsis overflow-hidden`}>
              {props.kicker}
            </Text>
          </Show>
          <Text class={`font-display text-base font-bold leading-tight text-fg line-clamp-2 m-0 overflow-hidden`} label={props.title}>
            {props.title}
          </Text>
        </View>
        <Show when={props.gold}>
          <View class={`flex-row items-center pr-2 pl-0.5 py-0.5 gap-1 ${packClass("Chip", "frame")}`} label={t("rpg.backpack.gold", { gold: props.gold })}>
            <Icon class="size-5.5 text-gold" name="coin" />
            <Text class={`tabular-nums whitespace-nowrap font-bold shrink`}>
              {props.gold}
            </Text>
          </View>
        </Show>
      </View>
      <Show when={props.tabs.length}>
        <View class="flex-row items-center w-full gap-1" onAction={{ tabNext: (e) => props.onTabNext?.(e), tabPrev: (e) => props.onTabPrev?.(e) }}>
          <Text glyph={"tabPrev"} class={keyCap} />
          <Scroll class="grow shrink basis-0" scrollbar={false}>
            <Tabs label={t("rpg.backpack.tabs")}>
              <For each={props.tabs} key={(c) => c.id}>
                {(c) => (
                <Tab class="shrink-0 px-1.5 gap-1" selected={c().id === props.tab} onPress={(e) => props.onChoosetab?.(e, c().id)}>
                  <Text class="whitespace-nowrap">{c().label}</Text>
                  <Show when={c().count > 0}>
                    <Text class={packClass("Tab", "count")}>{formatNumber(c().count)}</Text>
                  </Show>
                </Tab>
                )}
              </For>
            </Tabs>
          </Scroll>
          <Text glyph={"tabNext"} class={keyCap} />
        </View>
      </Show>
      <Show when={props.capacity}>
        <Text class={`text-caption text-fg-muted px-1 m-0`}>
          {props.capacity}
        </Text>
      </Show>
      <Show when={props.items.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Icon class="size-12" name="bag" />
            <Text class={`text-caption text-fg-muted m-0`}>
              {t("rpg.backpack.empty")}
            </Text>
          </View>
      }>
        <View class="grow shrink basis-0 gap-2 overflow-auto">
          <View class="grid grid-cols-fill-18 w-full p-1 gap-0.5 compact:grid-cols-5" role="grid" label={props.title}>
            <For each={props.items} key={(i) => i.id}>
              {(i) => (
              <View class={`group/cell text-fg transition duration-control w-full aspect-square min-w-11 p-0.5 data-down:opacity-dim`} role="gridcell" selected={i().id === props.selected} label={i().name} onPress={(e) => props.onChoose?.(e, i().id)} draggable={i().locked ? undefined : i().id} dropTarget={i().id} onDrop={(e) => { if (e.key !== i().id) props.onMove?.(e, { dragged: e.key, target: i().id }); }} data-selected={i().id === props.selected} data-broken={i().broken} data-tier-uncommon={(i().rarity || 'common') === "uncommon"} data-tier-rare={(i().rarity || 'common') === "rare"} data-tier-epic={(i().rarity || 'common') === "epic"} data-tier-legendary={(i().rarity || 'common') === "legendary"} data-tier-mythic={(i().rarity || 'common') === "mythic"}>
                <View class={`relative grow shrink basis-0 justify-center items-center w-full gap-0 overflow-hidden hover:inset-ring-border-strong group-data-tier-uncommon/cell:inset-ring-regular group-data-tier-uncommon/cell:inset-ring-uncommon group-data-tier-rare/cell:inset-ring-regular group-data-tier-rare/cell:inset-ring-rare group-data-tier-epic/cell:inset-ring-regular group-data-tier-epic/cell:inset-ring-epic group-data-tier-legendary/cell:inset-ring-regular group-data-tier-legendary/cell:inset-ring-legendary group-data-tier-mythic/cell:inset-ring-regular group-data-tier-mythic/cell:inset-ring-mythic group-data-selected/cell:inset-ring-regular group-data-selected/cell:inset-ring-hud-accent ${packClass("Slot", "frame")}`}>
                  <Icon class="size-17/25" name={i().icon} />
                  <View class="absolute bottom-0 h-0.75 bg-current inset-x-2 text-common group-data-tier-uncommon/cell:text-uncommon group-data-tier-rare/cell:text-rare group-data-tier-epic/cell:text-epic group-data-tier-legendary/cell:text-legendary group-data-tier-mythic/cell:text-mythic" />
                  <Show when={i().broken}>
                    <Text class={`absolute bottom-1 text-center text-xs font-bold text-danger whitespace-nowrap inset-x-0`}>
                      {t("rpg.backpack.broken")}
                    </Text>
                  </Show>
                </View>
                <Show when={i().count > 1}>
                  <Text class={`font-bold text-xs leading-snug tabular-nums transition duration-control absolute whitespace-nowrap right-1 bottom-2 py-0 px-0.5 group-data-broken/cell:bottom-7.5 ${packClass("Chip", "frame")}`} label={formatNumber(i().count)}>
                    {formatCompact(i().count)}
                  </Text>
                </Show>
                <Show when={i().equipped}>
                  <Text class={`absolute text-xs font-bold leading-snug text-success-fg bg-success whitespace-nowrap left-1 top-1 py-0 px-1 rounded-control`} label={t("rpg.backpack.equipped")}>
                    {t("rpg.backpack.equippedMark")}
                  </Text>
                </Show>
                <Show when={i().locked}>
                  <View class={`absolute items-center justify-center right-1 top-1 p-0 size-5 ${packClass("Chip", "frame")}`} label={t("rpg.backpack.locked")}>
                    <Icon class="size-16/25" name="lock" />
                  </View>
                </Show>
                <Show when={i().fresh}>
                  <View class={`bg-hud-accent absolute right-2 top-2 shadow-glow-sm ring-regular ring-shade-deep size-1 rounded-capsule data-locked:right-8.5`} label={t("rpg.backpack.new")} data-locked={i().locked} />
                </Show>
                <Show when={i().id === props.selected}>
                  <View class="absolute pointer-events-none inset-0">
                    <View class="absolute left-0 top-0 w-3 h-0.5 bg-fg" />
                    <View class="absolute left-0 top-0 w-0.5 h-3 bg-fg" />
                    <View class="absolute right-0 top-0 w-3 h-0.5 bg-fg" />
                    <View class="absolute right-0 top-0 w-0.5 h-3 bg-fg" />
                    <View class="absolute left-0 bottom-0 w-3 h-0.5 bg-fg" />
                    <View class="absolute left-0 bottom-0 w-0.5 h-3 bg-fg" />
                    <View class="absolute right-0 bottom-0 w-3 h-0.5 bg-fg" />
                    <View class="absolute right-0 bottom-0 w-0.5 h-3 bg-fg" />
                  </View>
                </Show>
              </View>
              )}
            </For>
          </View>
        </View>
      </Show>
      <Show when={props.weight}>
        <View class="w-full pt-2 pb-0 border-t border-border-muted px-1 gap-1 compact:pt-1">
          <View class="flex-row items-baseline gap-2">
            <Text class={`grow shrink basis-0 text-caption text-fg-muted tracking-kicker uppercase`}>
              {t("rpg.backpack.weight")}
            </Text>
            <Text class={`tabular-nums text-fg whitespace-nowrap font-bold data-over:text-warn shrink`} data-over={props.load > 1}>
              {props.weight}
            </Text>
          </View>
          <View class={`bg-track shadow-sunk h-2.5 mx-1 rounded-capsule data-over:shadow-glow-lg data-over:shadow-warn data-over:ring data-over:ring-warn`} role="meter" label={t("rpg.backpack.weight")} valueText={props.weight} data-over={props.load > 1}>
            <View class={`bg-hud-accent absolute left-0 inset-y-0 data-over:bg-none data-over:bg-warn`} data-over={props.load > 1} style={{ width: `${(100 * (props.load > 1 ? 1 : props.load))}%` }} />
          </View>
          <Show when={props.load > 1}>
            <Text class={`text-caption text-warn m-0`}>
              {t("rpg.backpack.over")}
            </Text>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgBackpackProps>> = {
  default: {"title":"Backpack","kicker":"Inventory","tab":"all","tabs":[{"id":"all","label":"All","count":6},{"id":"gear","label":"Weapons","count":2},{"id":"armour","label":"Armour","count":1},{"id":"use","label":"Consumables","count":2},{"id":"mat","label":"Materials","count":1}],"items":[{"id":"blade","name":"Emberwake Blade","icon":"sword","count":1,"rarity":"legendary","equipped":true,"fresh":false,"locked":false,"broken":false},{"id":"potion","name":"Healing Draught","icon":"potion","count":5,"rarity":"common","equipped":false,"fresh":true,"locked":false,"broken":false},{"id":"shield","name":"Warden Shield","icon":"shield","count":1,"rarity":"epic","equipped":false,"fresh":false,"locked":true,"broken":false},{"id":"crown","name":"Cracked Circlet","icon":"crown","count":1,"rarity":"rare","equipped":false,"fresh":false,"locked":false,"broken":true},{"id":"gem","name":"Storm Crystal","icon":"gem","count":12,"rarity":"uncommon","equipped":false,"fresh":false,"locked":false,"broken":false},{"id":"key","name":"Vault Key","icon":"key","count":1,"rarity":"mythic","equipped":false,"fresh":false,"locked":false,"broken":false}],"selected":"blade","capacity":"6 / 36 slots","weight":"92.4 / 120","load":0.77,"gold":"4,327"},
  empty: {"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All","count":0}],"items":[],"selected":"","weight":"0 / 120","load":0,"gold":"0"},
  over: {"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All","count":2}],"items":[{"id":"tower","name":"Tower Shield","icon":"shield","count":1,"rarity":"common","equipped":false,"fresh":false,"locked":false,"broken":false},{"id":"ore","name":"Iron Ore","icon":"gem","count":40,"rarity":"common","equipped":false,"fresh":false,"locked":false,"broken":false}],"selected":"","weight":"131.0 / 120","load":1.09,"gold":"12"},
  overflow: {"title":"The Bottomless Satchel of the Wandering Quartermaster","kicker":"Inventory","tab":"mat","tabs":[{"id":"all","label":"All","count":1240},{"id":"gear","label":"Weapons","count":312},{"id":"armour","label":"Armour","count":208},{"id":"acc","label":"Accessories","count":96},{"id":"use","label":"Consumables","count":420},{"id":"mat","label":"Crafting materials","count":204}],"items":[{"id":"a","name":"Ore","icon":"gem","count":99999,"rarity":"common","equipped":false,"fresh":true,"locked":true,"broken":false},{"id":"b","name":"Blade","icon":"sword","count":1,"rarity":"mythic","equipped":true,"fresh":true,"locked":true,"broken":true}],"selected":"b","capacity":"1,240 / 1,500 slots","weight":"1,234.5 / 2,000","load":0.62,"gold":"1,234,567,890"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"tabPrev":"Previous category","tabNext":"Next category"} as const;
