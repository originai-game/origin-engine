import { For, Icon, Show, Text, View, formatCompact, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import "./backpack.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] font-bold leading-[1.5] py-0 px-1`;

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
  /** The move event, with the row's key where it comes from a row. */
  onMove?: (e: UiEvent, key?: string) => void;
  /** The tabNext event, with the row's key where it comes from a row. */
  onTabNext?: (e: UiEvent, key?: string) => void;
  /** The tabPrev event, with the row's key where it comes from a row. */
  onTabPrev?: (e: UiEvent, key?: string) => void;
}

/**
 * A bag of items: a band with a kicker, the title and the purse, category
 * tabs between the action's key caps, how full the bag is, a grid of item
 * tiles and a footer with the carried weight. The game owns what a
 * category holds, the order and every number; the bag reports choose (an
 * item's key), choosetab (a category's key), tabNext and tabPrev. A tile
 * drags onto another (a locked one stays put) and the bag reports move
 * with { dragged, target }, the two items' keys; the dragged tile springs
 * home, and the game reorders, swaps or stacks as it sees fit. The grid
 * fills the height it is given and scrolls inside it. A tile shows its
 * rarity as its rim, a light rising from its foot and a bar along it; its
 * stack, Equipped, New, Locked and Broken sit in its corners, each named
 * in words for a screen reader, Broken also on the tile. The selected
 * tile wears corner brackets.
 */
export function RpgBackpack(props: RpgBackpackProps) {
  return (
    <View class={`grow shrink basis-0 border-rim rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-glass-hi via-glass via-[96px] to-glass-lo shadow-panel text-fg p-4 gap-2 border disabled:text-fg-subtle`} label={props.title}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3 compact:py-1`}>
        <View class="grow shrink basis-[8.571rem] gap-0.5">
          <Show when={props.kicker}>
            <Text class={`text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-subtle whitespace-nowrap text-ellipsis overflow-hidden`}>
              {props.kicker}
            </Text>
          </Show>
          <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg line-clamp-2 m-0 overflow-hidden`} label={props.title}>
            {props.title}
          </Text>
        </View>
        <Show when={props.gold}>
          <View class={`flex-row items-center pr-2 pl-0.5 bg-[#04060c8c] shadow-[inset_0px_0px_0px_1px_#4e586c] py-0.5 gap-1 rounded-capsule`} label={t("rpg.backpack.gold", { gold: props.gold })}>
            <Text class="size-[1.571rem] rounded-capsule" />
            <Text class={`tabular-nums text-fg whitespace-nowrap font-bold shrink`}>
              {props.gold}
            </Text>
          </View>
        </Show>
      </View>
      <Show when={props.tabs.length}>
        <View class="flex-row items-center w-full border-b border-rim-soft gap-1" role="tablist" label={t("rpg.backpack.tabs")} onAction={{ tabNext: (e) => props.onTabNext?.(e), tabPrev: (e) => props.onTabPrev?.(e) }}>
          <Text glyph={"tabPrev"} class={keyCap} />
          <View class="grow shrink basis-0 overflow-x-auto overflow-y-hidden">
            <View class="flex-row items-center pr-12 gap-0">
              <For each={props.tabs}>
                {(c) => (
                <View class={`group/tab text-fg-muted transition duration-317 min-h-11 py-0 px-2 gap-1 data-selected:text-fg`} role="tab" selected={c().id === props.tab} onPress={(e) => props.onChoosetab?.(e, c().id)} data-selected={c().id === props.tab}>
                  <Text class="whitespace-nowrap">{c().label}</Text>
                  <Show when={c().count > 0}>
                    <Text class={`text-[0.857rem] font-bold bg-material-edge text-fg-subtle py-0 px-1 rounded-capsule group-data-selected/tab:bg-none group-data-selected/tab:bg-[#38e8ff] group-data-selected/tab:text-[#041016]`}>
                      {formatNumber(c().count)}
                    </Text>
                  </Show>
                  <Text class="absolute bottom-0 h-0.75 bg-[#38e8ff] shadow-[0px_0px_11.424px_#38e8ff] opacity-0 inset-x-1 group-data-selected/tab:opacity-100" />
                </View>
                )}
              </For>
            </View>
          </View>
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
          <View class="grid grid-cols-[repeat(auto-fill,minmax(calc(72_*_calc(1rem_/_14)),1fr))] w-full p-1 gap-0.5 compact:grid-cols-5" role="grid" label={props.title}>
            <For each={props.items}>
              {(i) => (
              <View class={`group/cell text-fg transition duration-317 w-full aspect-square min-w-11 p-0.5 data-selected:border-accent data-done:border-success data-active:border-accent hover:bg-none hover:bg-transparent hover:shadow-none disabled:text-fg-subtle disabled:scale-100 data-broken:text-danger data-broken:absolute data-broken:bottom-1 data-broken:text-center data-broken:text-[0.857rem] data-broken:font-bold data-broken:inset-x-0 active:scale-96 data-locked:scale-100 data-down:opacity-45`} data-equipped={i().equipped} data-fresh={i().fresh} role="gridcell" selected={i().id === props.selected} label={i().name} draggable={!i().locked} onPress={(e) => props.onChoose?.(e, i().id)} onAction={{ drop: (e) => props.onMove?.(e, i().id) }} data-broken={i().broken} data-selected={i().id === props.selected} data-locked={i().locked}>
                <View class="grow shrink basis-0 justify-center items-center w-full bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[9.136px] rounded-tr-none rounded-br-[9.136px] rounded-bl-none gap-0">
                  <Icon class="size-[68%]" name={i().icon} />
                  <View class="absolute bottom-0 h-0.75 text-border-strong bg-current inset-x-[0.571rem]" />
                  <Show when={i().broken}>
                    <Text class={`absolute bottom-1 text-center text-[0.857rem] font-bold text-danger whitespace-nowrap inset-x-0`}>
                      {t("rpg.backpack.broken")}
                    </Text>
                  </Show>
                </View>
                <Show when={i().count > 1}>
                  <Text class={`font-bold text-[0.857rem] leading-[1.35] tabular-nums rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff33] text-fg transition duration-317 absolute whitespace-nowrap right-1 bottom-2 py-0 px-0.5 group-data-broken/cell:bottom-[29.943px]`} label={formatNumber(i().count)}>
                    {formatCompact(i().count)}
                  </Text>
                </Show>
                <Show when={i().equipped}>
                  <Text class={`absolute text-[0.857rem] font-bold leading-[1.35] text-[#04100a] bg-success rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none whitespace-nowrap left-1 top-1 py-0 px-1`} label={t("rpg.backpack.equipped")}>
                    {t("rpg.backpack.equippedMark")}
                  </Text>
                </Show>
                <Show when={i().locked}>
                  <View class="grow shrink basis-0 justify-center items-center absolute text-[0.857rem] font-bold leading-[1.35] text-fg bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff33] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none right-1 top-1 p-0 gap-2 size-[1.429rem]" label={t("rpg.backpack.locked")}>
                    <Icon class="size-[64%]" name="lock" />
                  </View>
                </Show>
                <Show when={i().fresh}>
                  <View class={`bg-[#38e8ff] text-fg absolute right-2 top-2 shadow-[0px_0px_9.136px_#38e8ff,0px_0px_0px_1.5px_#060910e5] size-1 rounded-capsule data-locked:right-[33.714px]`} label={t("rpg.backpack.new")} data-locked={i().locked} />
                </Show>
                <Show when={i().id === props.selected}>
                  <View class={`absolute pointer-events-none inset-0`} />
                </Show>
              </View>
              )}
            </For>
          </View>
        </View>
      </Show>
      <Show when={props.weight}>
        <View class="w-full pt-2 pb-0 border-t border-rim-soft px-1 gap-1 compact:pt-1">
          <View class="flex-row items-baseline gap-2">
            <Text class={`grow shrink basis-0 text-caption text-fg-muted tracking-[0.18em] uppercase`}>
              {t("rpg.backpack.weight")}
            </Text>
            <Text class={`tabular-nums text-fg whitespace-nowrap font-bold data-tone-warn:text-warn data-over:m-0 shrink`} data-over={props.load > 1} data-tone-warn={((props.load > 1 ? 'warn' : '') === 'warn') || ((props.load > 1 ? 'warn' : '') === "warn")}>
              {props.weight}
            </Text>
          </View>
          <View class={`bg-tile shadow-sunk h-[0.714rem] mx-1 rounded-capsule data-over:shadow-[0px_0px_0px_1px_#ffa53d,0px_0px_13.712px_#ffa53d99] data-over:m-0`} role="meter" label={t("rpg.backpack.weight")} valueText={props.weight} data-over={props.load > 1}>
            <View class={`bg-[#38e8ff] absolute left-0 inset-y-0 data-tone-warn:bg-none data-tone-warn:bg-warn data-tone-warn:text-warn`} data-tone-warn={((props.load > 1 ? 'warn' : '') === 'warn') || ((props.load > 1 ? 'warn' : '') === "warn")} style={{ width: `${(100 * (props.load > 1 ? 1 : props.load))}%` }} />
          </View>
          <Show when={props.load > 1}>
            <Text class={`text-caption text-warn m-0`} data-tone-warn={true}>
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
