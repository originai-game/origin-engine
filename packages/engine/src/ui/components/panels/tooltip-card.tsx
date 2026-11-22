import { For, Icon, Image, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./tooltip-card.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden text-xl leading-none`;

/** One stat row of either card: its words left, its figure right, a hairline rule under all but the last. */
const statRow =
  "group/row flex-row flex-wrap items-baseline gap-y-0.5 gap-x-2 text-fg transition duration-317 w-full justify-end border-b border-material-edge py-1 px-0 data-last:border-b-0 data-change-removed:text-fg-subtle data-down:opacity-45";

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

/**
 * An item's tooltip card: a dense smoked plate lit along its top in its
 * rarity's colour, its icon in a socket of that colour, the title and the
 * one-to-six rarity pips, its stats (label, figure, a signed delta in the
 * better green or the worse red with its triangle, an added or removed
 * stat a chip), the body, a footer line and the action hints with their
 * key caps; what is equipped stands beside it on a plainer plate, and
 * under it on a narrow screen. A long name or stat wraps between words.
 * Read only.
 *
 * Dropped against the .svelte: the socket's octagon (no clip-path here; a
 * square well stands in) and the CJK heading sizes (a lang has no class
 * variant here).
 */
export function TooltipCard(props: TooltipCardProps) {
  const tier = () => props.rarity;
  const tiered = () =>
    tier() === "common" || tier() === "uncommon" || tier() === "rare" || tier() === "epic" || tier() === "legendary" || tier() === "mythic";
  return (
    <View class="flex-row flex-wrap items-start max-w-full text-fg gap-2 compact:flex-col-reverse compact:flex-nowrap compact:items-stretch" role="tooltip" label={props.title} description={props.body}>
      <Show when={props.compareTitle !== ""}>
        <View class={`bg-linear-to-b from-raised-pressed to-plate-lo inset-shadow-lit inset-ring-1 inset-ring-rim text-fg w-75 max-w-full py-3 px-4 gap-2 compact:w-auto shrink ${packClass("Panel", "frame")}`}>
          <Text class="font-semibold text-xs leading-snug text-fg-subtle tracking-label uppercase">
            {props.compareTitle}
          </Text>
          <View class="w-full h-px bg-rim m-0" role="separator" />
          <View class="gap-y-0 gap-x-0.5 w-full p-0 m-0">
            <For each={props.compareStats}>
              {(s, i) => (
                <View class={`${statRow}`} data-last={i() === props.compareStats.length - 1}>
                  <Text class="grow shrink basis-27.5 text-fg-muted text-base leading-snug">
                    {s().label}
                  </Text>
                  <Text class="tabular-nums text-fg text-base font-bold whitespace-nowrap shrink">
                    {s().value}
                  </Text>
                </View>
              )}
            </For>
          </View>
        </View>
      </Show>
      <View
        class={`group/card relative bg-linear-to-b from-raised-pressed to-plate-lo text-fg data-common:text-common data-uncommon:text-uncommon data-rare:text-rare data-epic:text-epic data-legendary:text-legendary data-mythic:text-mythic w-75 max-w-full py-3 px-4 gap-2 compact:w-auto shrink overflow-hidden ${packClass("Panel", "frame")}`}
        data-common={tier() === "common"}
        data-uncommon={tier() === "uncommon"}
        data-rare={tier() === "rare"}
        data-epic={tier() === "epic"}
        data-legendary={tier() === "legendary"}
        data-mythic={tier() === "mythic"}
      >
        <Show when={tiered()}>
          <View class="absolute inset-x-0 top-0 h-22.5 bg-linear-to-b from-current to-transparent opacity-16 pointer-events-none" />
          <View class="absolute left-0 right-0 top-0 h-0.5 bg-current pointer-events-none" />
        </Show>
        <View class="relative flex-row items-center gap-y-2 gap-x-3 w-full">
          <Show when={props.image || props.icon}>
            <View class="flex-row items-center justify-center gap-2 size-12 overflow-hidden bg-tile">
              <Show when={tiered()}>
                <View class="absolute inset-0 bg-current opacity-40" />
              </Show>
              <Show when={props.image} fallback={
                <Icon class="relative text-fg size-3/5 shrink" name={props.icon} />
              }>
                <Image class="relative size-full shrink" src={props.image} />
              </Show>
            </View>
          </Show>
          <View class="grow shrink basis-0 gap-0.5">
            <Text class="font-display text-lg font-bold leading-snug m-0">
              {props.title}
            </Text>
            <Show when={props.subtitle}>
              <View class="flex-row flex-wrap items-center gap-y-0.5 gap-x-2 mt-0.5">
                <View class="flex-row items-center gap-1 shrink">
                  <Show when={tier()}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                  <Show when={tier() && tier() !== "common"}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                  <Show when={tier() === "rare" || tier() === "epic" || tier() === "legendary" || tier() === "mythic"}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                  <Show when={tier() === "epic" || tier() === "legendary" || tier() === "mythic"}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                  <Show when={tier() === "legendary" || tier() === "mythic"}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                  <Show when={tier() === "mythic"}>
                    <View class="bg-current rotate-45 size-1.5" />
                  </Show>
                </View>
                <Text class="font-semibold text-xs leading-snug text-fg-muted tracking-label uppercase shrink">
                  {props.subtitle}
                </Text>
              </View>
            </Show>
          </View>
        </View>
        <View class="w-full h-px bg-rim m-0 relative" role="separator" />
        <Show when={props.stats.length}>
          <View class="gap-y-0 gap-x-0.5 w-full p-0 m-0">
            <For each={props.stats}>
              {(s, i) => (
                <View class={statRow} data-last={i() === props.stats.length - 1} data-change-removed={(s().change) === "removed"}>
                  <Text class="grow shrink basis-27.5 text-fg-muted text-base leading-snug">
                    {s().label}
                  </Text>
                  <Text class="tabular-nums text-fg text-base font-bold whitespace-nowrap shrink">
                    {s().value}
                  </Text>
                  <Show when={s().change === "added" || s().change === "removed"} fallback={
                    <Show when={(s().delta ?? 0) !== 0}>
                      <View class="flex-row items-center min-w-13 justify-end text-base font-bold gap-0.5 data-good:text-success data-bad:text-danger shrink" data-good={s().better ? (s().delta ?? 0) < 0 : (s().delta ?? 0) > 0} data-bad={s().better ? (s().delta ?? 0) > 0 : (s().delta ?? 0) < 0}>
                        <Icon name="play" class="size-2.25 -rotate-90 data-bad:rotate-90" data-bad={(s().delta ?? 0) < 0} />
                        <Text class="tabular-nums whitespace-nowrap shrink">
                          {s().deltaText}
                        </Text>
                      </View>
                    </Show>
                  }>
                    <Text class={`font-bold text-2xs leading-snug text-success tracking-label whitespace-nowrap bg-well py-0.5 px-1 group-data-change-removed/row:text-fg-subtle ${packClass("Chip", "frame")}`}>
                      {s().change === "added" ? t("tooltip.change.added") : t("tooltip.change.removed")}
                    </Text>
                  </Show>
                </View>
              )}
            </For>
          </View>
        </Show>
        <Show when={props.body}>
          <Text class="text-base leading-normal text-fg-muted m-0">{props.body}</Text>
        </Show>
        <Show when={props.footer}>
          <Text class="text-caption leading-snug text-fg-subtle m-0">
            {props.footer}
          </Text>
        </Show>
        <Show when={props.hints.length}>
          <View class="flex-row flex-wrap items-center gap-y-1 gap-x-3 w-full pt-2 pb-0 border-t border-sheen px-0 m-0">
            <For each={props.hints}>
              {(h) => (
                <View class="flex-row items-center font-semibold text-caption text-fg transition duration-317 p-0 gap-1 shrink">
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
