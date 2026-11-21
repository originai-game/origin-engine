import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./tooltip-card.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] font-bold leading-[1.5] py-0 px-1`;

export interface TooltipCardProps {
  /** The title, in its rarity's colour. */
  title: string;
  /** A line under it (Legendary sword, Passive); empty: none. */
  subtitle: string;
  /** none, common, uncommon, rare, epic, legendary or mythic: the title's and frame's colour and its pips (one to six, so no tier is told by colour alone). */
  rarity: string;
  /** An icon beside the title when there is no image; empty: none. */
  icon: string;
  /** An art slot beside the title (an item or ability texture handle, 1:1); 0: the icon. */
  image: Texture | 0;
  /** The body text; empty: none. */
  body: string;
  /** Keyed { id, label, value, delta, deltaText, better, change? } comparison facts supplied by the game. Signed delta and its arrow describe the numeric direction; better true means lower is better, flipping the colours. Optional change is added or removed; absent means both items have the stat. Added shows value without delta; removed shows the compared value muted. Presence is translated and exposed as data-change, never inferred from strings. */
  stats: ReadonlyArray<{ id: string; label: string; value: string; delta?: number; deltaText?: string; better?: boolean; change?: string }>;
  /** The heading over a second card of what is equipped (Equipped); empty draws no second card. */
  compareTitle: string;
  /** The equipped item's stat rows, { id, label, value }. */
  compareStats: ReadonlyArray<{ id: string; label: string; value: string }>;
  /** A last line (Sell for 120, a flavour quote) in the muted ink; empty: none. */
  footer: string;
  /** Action hints (Right-click to equip), a keyed collection of { id, text, action }: action an input action whose glyph leads the hint; empty: none. */
  hints: ReadonlyArray<{ id: string; text: string; action: string }>;
}

export function TooltipCard(props: TooltipCardProps) {
  return (
    <View class="flex-row flex-wrap items-start max-w-full text-accent-hover gap-2 compact:flex-col-reverse compact:flex-nowrap compact:items-stretch" role="tooltip" label={props.title} description={props.body}>
      <Show when={props.compareTitle !== ""}>
        <View class={`bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff1f,inset_0px_1px_0px_#ffffff66] text-accent-hover w-[21.429rem] max-w-full py-3 px-4 gap-2 disabled:text-fg-subtle compact:w-auto shrink`}>
          <Text class={`font-semibold text-[0.857rem] leading-snug text-accent-hover/56 tracking-[0.14em] uppercase`}>
            {props.compareTitle}
          </Text>
          <View class="w-full text-sheen m-0" />
          <View class="gap-y-0 gap-x-0.5 w-full p-0 m-0">
            <For each={props.compareStats}>
              {(s, i) => (
              <View class={`flex-row flex-wrap items-baseline gap-y-0.5 gap-x-2 text-accent-hover transition duration-317 w-full justify-end border-b border-material-edge py-1 px-0 data-selected:text-fg data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 data-last:border-b-0`} data-last={i() === props.compareStats.length - 1}>
                <Text class={`grow shrink basis-[7.857rem] text-accent-hover/74 text-base leading-snug`}>
                  {s().label}
                </Text>
                <Text class={`tabular-nums text-accent-hover text-[1.143rem] font-bold whitespace-nowrap shrink`}>
                  {s().value}
                </Text>
              </View>
              )}
            </For>
          </View>
        </View>
      </Show>
      <View class={`group/card text-accent-hover w-[21.429rem] max-w-full py-3 px-4 gap-2 disabled:text-fg-subtle compact:w-auto shrink`} data-common={props.rarity === "common"} data-uncommon={props.rarity === "uncommon"} data-rare={props.rarity === "rare"} data-epic={props.rarity === "epic"} data-legendary={props.rarity === "legendary"} data-mythic={props.rarity === "mythic"} data-unranked={props.rarity !== "common" && props.rarity !== "uncommon" && props.rarity !== "rare" && props.rarity !== "epic" && props.rarity !== "legendary" && props.rarity !== "mythic"}>
        <View class="flex-row items-center gap-y-2 gap-x-3 w-full">
          <Show when={props.image || props.icon}>
            <View class={`flex-row items-center justify-center gap-2 size-[3.429rem] overflow-hidden`}>
              <Show when={props.image} fallback={
                  <Icon class="text-accent-hover size-3/5 shrink" name={props.icon} />
              }>
                <Image class="size-full shrink" src={props.image} />
              </Show>
            </View>
          </Show>
          <View class="grow shrink basis-0 gap-0.5">
            <Text class={`font-display text-[1.357rem] font-bold leading-snug m-0 group-data-unranked/card:text-accent-hover`}>
              {props.title}
            </Text>
            <Show when={props.subtitle}>
              <View class="flex-row flex-wrap items-center gap-y-0.5 gap-x-2 mt-0.5">
                <View class="flex-row items-center gap-1 shrink">
                  <Show when={props.rarity !== 'none' && props.rarity !== ''}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                  <Show when={props.rarity !== 'none' && props.rarity !== '' && props.rarity !== 'common'}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                  <Show when={props.rarity === 'rare' || props.rarity === 'epic' || props.rarity === 'legendary' || props.rarity === 'mythic'}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                  <Show when={props.rarity === 'epic' || props.rarity === 'legendary' || props.rarity === 'mythic'}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                  <Show when={props.rarity === 'legendary' || props.rarity === 'mythic'}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                  <Show when={props.rarity === 'mythic'}>
                    <Text class={`text-fg rotate-45 size-[0.429rem]`} />
                  </Show>
                </View>
                <Text class={`font-semibold text-[0.857rem] leading-snug text-accent-hover/74 tracking-[0.14em] uppercase shrink`}>
                  {props.subtitle}
                </Text>
              </View>
            </Show>
          </View>
        </View>
        <View class="w-full text-sheen m-0" />
        <Show when={props.stats.length}>
          <View class="gap-y-0 gap-x-0.5 w-full p-0 m-0">
            <For each={props.stats}>
              {(s, i) => (
              <View class={`group/row flex-row flex-wrap items-baseline gap-y-0.5 gap-x-2 text-accent-hover transition duration-317 w-full justify-end border-b border-material-edge py-1 px-0 data-selected:text-fg data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 data-last:border-b-0`} data-last={i() === props.stats.length - 1} data-change-removed={(s().change) === 'removed'}>
                <Text class={`grow shrink basis-[7.857rem] text-accent-hover/74 text-base leading-snug group-data-change-removed/row:text-accent-hover/56`}>
                  {s().label}
                </Text>
                <Text class={`tabular-nums text-accent-hover text-[1.143rem] font-bold whitespace-nowrap group-data-change-removed/row:text-accent-hover/56 shrink`}>
                  {s().value}
                </Text>
                <Show when={s().change === 'added' || s().change === 'removed'} fallback={
                    <Show when={s().delta !== 0}>
                      <View class={`flex-row items-center min-w-[3.714rem] justify-end text-base font-bold gap-0.5 data-trend-up:text-success data-trend-down:text-danger data-good:text-success data-bad:text-danger shrink`} data-good={s().better ? s().delta < 0 : s().delta > 0} data-bad={s().better ? s().delta > 0 : s().delta < 0} data-trend-up={((s().better ? s().delta < 0 : s().delta > 0) ? 'up' : 'down') === "up"} data-trend-down={((s().better ? s().delta < 0 : s().delta > 0) ? 'up' : 'down') === "down"}>
                        <Icon name="play" class="size-[0.643rem] -rotate-90 data-down:rotate-90" data-down={(s().delta ?? 0) < 0} />
                        <Text class={`tabular-nums whitespace-nowrap shrink`}>
                          {s().deltaText}
                        </Text>
                      </View>
                    </Show>
                }>
                  <Text class={`font-bold text-[0.714rem] leading-snug text-success tracking-[0.14em] whitespace-nowrap bg-[#060910e0] py-0.5 px-1 group-data-change-removed/row:text-accent-hover/56`}>
                    {s().change === 'added' ? t("tooltip.change.added") : t("tooltip.change.removed")}
                  </Text>
                </Show>
              </View>
              )}
            </For>
          </View>
        </Show>
        <Show when={props.body}>
          <Text class="text-base leading-normal text-accent-hover/74 m-0">{props.body}</Text>
        </Show>
        <Show when={props.footer}>
          <Text class={`text-[0.929rem] leading-snug text-accent-hover/56 m-0`}>
            {props.footer}
          </Text>
        </Show>
        <Show when={props.hints.length}>
          <View class="flex-row flex-wrap items-center gap-y-1 gap-x-3 w-full pt-2 pb-0 border-t border-sheen px-0 m-0">
            <For each={props.hints}>
              {(h) => (
              <View class={`flex-row items-center font-semibold text-[0.929rem] leading-4.5 text-accent-hover transition duration-317 p-0 gap-1 shrink`}>
                <Show when={h().action}>
                  <Text glyph={h().action} class={keyCap} />
                </Show>
                <Text class="whitespace-nowrap tabular-nums shrink">{h().text}</Text>
              </View>
              )}
            </For>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<TooltipCardProps>> = {
  default: {"title":"Ember Blade","subtitle":"Legendary sword","rarity":"legendary","icon":"sword","image":0,"body":"Burns on hit for 12 fire damage over 3 seconds. Two-handed.","stats":[{"id":"dmg","label":"Damage","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"speed","label":"Attack speed","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"cd","label":"Cooldown","value":"4 s","delta":0,"deltaText":"","better":true}],"compareTitle":"Equipped","compareStats":[{"id":"dmg","label":"Damage","value":"30–46"},{"id":"speed","label":"Attack speed","value":"1.3"},{"id":"cd","label":"Cooldown","value":"4 s"}],"footer":"Sell for 120","hints":[{"id":"equip","text":"Equip","action":"confirm"}]},
  comparison: {"title":"The Ember Blade of the Last Watcher of the Ninefold Gate","stats":[{"id":"damage","label":"Damage against expedition guardians","value":"58","delta":12,"deltaText":"+12","better":false},{"id":"cooldown","label":"Recovery time after a charged attack","value":"3 s","delta":-1,"deltaText":"−1 s","better":true},{"id":"fire","label":"Resistance to burning ground in the northern territories","value":"8%","change":"added"},{"id":"poison","label":"Resistance to poison from creatures of the Sunken Kingdom","value":"12%","change":"removed"}],"compareTitle":"Equipped","compareStats":[{"id":"damage","label":"Damage against expedition guardians","value":"46"},{"id":"cooldown","label":"Recovery time after a charged attack","value":"4 s"},{"id":"poison","label":"Resistance to poison from creatures of the Sunken Kingdom","value":"12%"}]},
};
