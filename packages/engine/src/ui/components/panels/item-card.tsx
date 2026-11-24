import { For, Icon, Image, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Button } from "../../controls/button";
import "./item-card.en.ogt";

/** A binding's key cap: hidden on touch, where no key is pressed. */
const keyCap = `shrink-0 pointer-coarse:hidden text-xl leading-none`;

/** A change chip: the delta in the trend's colour with its triangle; plain (no trend) it is muted type with no ground. */
const deltaChip =
  "group/delta flex-row items-center font-bold text-fg-muted py-0 px-1 gap-0.5 data-trend-up:text-success-fg data-trend-up:bg-success data-trend-down:text-danger-fg data-trend-down:bg-danger";

export interface RpgItemCardProps {
  /** The item's name. */
  name: string;
  /** Its icon in the style's or the game's atlas, drawn large at the top. */
  icon: string;
  /** A painted picture of the item (og.load.texture) shown large in place of its icon; 0 shows the icon. */
  art: Texture | 0;
  /** common, uncommon, rare, epic, legendary or mythic; the name and the art take its ink. */
  rarity: string;
  /** The rarity and kind line as the game words it (Legendary · Longsword), a kicker over the name in the rarity's ink. */
  kind: string;
  /** A line under the name as the game words it (One-handed · Item level 60); empty hides it. */
  line?: string;
  /** The headline figure, zero or one row of { id, icon, label, value, delta, trend, current }: trend is up, down or none; delta and current are text, empty hides them. */
  lead: ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>;
  /** The other figures, keyed rows of { id, icon, label, value, delta, trend, current }. */
  stats: ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>;
  /** The flavour text; empty hides it. */
  lore: string;
  /** The main button's text (Equip, Use); empty hides it. */
  primary: string;
  /** The second button's text (Drop, Sell); empty hides it. */
  secondary: string;
  /** The main button is enabled. */
  canPrimary?: boolean;
  /** The second button is enabled. */
  canSecondary?: boolean;
  /** Why the main button is refused, as the game words it (Requires level 40); shown in red over the buttons while canPrimary is false. */
  reason?: string;
  /** The discard event, with the row's key where it comes from a row. */
  onDiscard?: (e: UiEvent, key?: string) => void;
  /** The equip event, with the row's key where it comes from a row. */
  onEquip?: (e: UiEvent, key?: string) => void;
}

/**
 * One item in detail: its art large over a glow of its rarity, the rarity
 * and kind as a kicker in the rarity's ink over the name, the headline
 * figure large with its change against what is equipped, the other figures
 * as rows (this, change, now), the lore under a rule of the rarity, and up
 * to two buttons, with the reason the main one is refused in words. The
 * game words every figure and decides the trend; the card reports equip
 * (the main button, X or pad Y) and discard (the second button). It fills
 * the height it is given: the figures and the lore scroll when needed; the
 * buttons stay at its foot; the art and the name scroll with the figures.
 * Narrow, the art sits in a socket beside the name and the rows drop the
 * equipped column.
 *
 * Dropped against the .svelte: the art's drop-shadow filter and the hero
 * glow's radial shape (a linear wash stands in; filters and radial
 * gradients have no surface here).
 */
export function RpgItemCard(props: RpgItemCardProps) {
  const tier = () => props.rarity || "common";
  const tiered = () => tier() !== "common";
  return (
    <View
      class={`group/card relative grow shrink basis-0 min-h-0 p-4 gap-2 disabled:text-fg-subtle ${packClass("Panel", "frame")}`}
      label={props.name}
      onAction={{ details: (e) => props.onEquip?.(e) }}
      data-tier-common={tier() === "common"}
      data-tier-uncommon={tier() === "uncommon"}
      data-tier-rare={tier() === "rare"}
      data-tier-epic={tier() === "epic"}
      data-tier-legendary={tier() === "legendary"}
      data-tier-mythic={tier() === "mythic"}
    >
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class="shrink w-full gap-1 overflow-auto">
        <View class="w-full gap-0 compact:flex-row compact:items-center compact:gap-2">
          <View class={`group/hero justify-center items-center w-full text-common group-data-tier-uncommon/card:text-uncommon group-data-tier-rare/card:text-rare group-data-tier-epic/card:text-epic group-data-tier-legendary/card:text-legendary group-data-tier-mythic/card:text-mythic border-b border-border-muted gap-2 compact:relative compact:grow-0 compact:shrink-0 compact:basis-auto compact:w-auto compact:border-0 compact:text-fg`}>
            <View class="absolute opacity-dim inset-0 bg-linear-to-b from-current to-transparent to-70% compact:hidden" />
            <Show when={props.art} fallback={
              <Icon class="relative size-[max(4rem,min(8.286rem,16vh))] compact:size-4/5" name={props.icon} />
            }>
              <Image class="relative object-contain size-[max(4rem,min(8.286rem,16vh))] compact:size-4/5" src={props.art} />
            </Show>
          </View>
          <View class={`min-h-12 pt-2 pb-1 border-b border-border-muted w-full px-4 gap-0.5 compact:min-w-0 compact:grow compact:shrink compact:basis-0 compact:p-0 compact:border-0`}>
            <Show when={props.kind}>
              <Text class="text-xs font-semibold tracking-kicker uppercase text-common group-data-tier-uncommon/card:text-uncommon group-data-tier-rare/card:text-rare group-data-tier-epic/card:text-epic group-data-tier-legendary/card:text-legendary group-data-tier-mythic/card:text-mythic compact:whitespace-nowrap compact:text-ellipsis compact:overflow-hidden" label={props.kind}>
                {props.kind}
              </Text>
            </Show>
            <Text class="text-fg text-shadow-xs text-shadow-border-strong group-data-tier-uncommon/card:text-uncommon group-data-tier-uncommon/card:text-shadow-uncommon group-data-tier-rare/card:text-rare group-data-tier-rare/card:text-shadow-rare group-data-tier-epic/card:text-epic group-data-tier-epic/card:text-shadow-epic group-data-tier-legendary/card:text-legendary group-data-tier-legendary/card:text-shadow-legendary group-data-tier-mythic/card:text-mythic group-data-tier-mythic/card:text-shadow-mythic text-xl m-0 compact:line-clamp-2 compact:overflow-hidden" label={props.name}>
              {props.name}
            </Text>
            <Show when={props.line}>
              <Text class="text-caption text-fg-muted m-0 compact:whitespace-nowrap compact:text-ellipsis compact:overflow-hidden" label={props.line}>
                {props.line}
              </Text>
            </Show>
          </View>
        </View>
        <For each={props.lead}>
          {(s) => (
            <View class="flex-row flex-wrap items-end rounded-card bg-layer-2 py-2 px-3 gap-x-3 gap-y-0.5" label={s().label}>
              <View class="grow gap-0.5">
                <Text class="text-xs font-semibold tracking-kicker uppercase text-fg-subtle">{s().label}</Text>
                <View class="flex-row items-center flex-wrap gap-1">
                  <Text class="tabular-nums text-fg text-display-lg leading-flat font-bold whitespace-nowrap compact:text-xl shrink">
                    {s().value}
                  </Text>
                  <Show when={s().delta}>
                    <View class={`${deltaChip} data-trend-up:nine-none data-trend-down:nine-none`} data-trend-down={(s().trend) === "down"} data-trend-up={(s().trend) === "up"}>
                      <Icon name="play" class="hidden bg-current size-2 group-data-trend-up/delta:flex -rotate-90 group-data-trend-down/delta:flex group-data-trend-down/delta:rotate-90" />
                      <Text class="whitespace-nowrap shrink">{s().delta}</Text>
                    </View>
                  </Show>
                </View>
              </View>
              <Show when={s().current}>
                <View class="ml-auto items-end text-end gap-0.5">
                  <Text class="text-xs font-semibold tracking-kicker uppercase text-fg-subtle">
                    {t("rpg.card.current")}
                  </Text>
                  <Text class="tabular-nums text-fg-muted text-lg font-bold whitespace-nowrap">
                    {s().current}
                  </Text>
                </View>
              </Show>
            </View>
          )}
        </For>
        <Show when={props.stats.length}>
          <View role="table" label={props.name}>
            <View class="flex-row items-center gap-y-0.5 gap-x-1 py-0.5 compact:hidden" role="row">
              <Text class="text-xs font-semibold tracking-kicker uppercase text-fg-subtle grow shrink basis-0 items-center pl-7" role="columnheader" label={t("rpg.card.stat")} />
              <View class="flex-row items-center ml-auto w-46.5 gap-1">
                <Text class="text-xs font-bold tracking-kicker uppercase text-fg-subtle w-auto text-end whitespace-nowrap min-w-11" role="columnheader">
                  {t("rpg.card.this")}
                </Text>
                <View class="w-auto grow shrink basis-0" />
                <Text class="text-xs font-semibold tracking-kicker uppercase text-fg-subtle w-auto text-end whitespace-nowrap shrink min-w-12 text-ellipsis overflow-hidden compact:hidden" role="columnheader">
                  {t("rpg.card.current")}
                </Text>
              </View>
            </View>
            <For each={props.stats}>
              {(s) => (
                <View class="flex-row items-center gap-y-0.5 gap-x-1 text-fg transition duration-control flex-wrap min-h-10 border-t border-border-muted py-0.5 hover:bg-surface-hover" role="row">
                  <View class="flex-row items-center grow shrink basis-20 text-fg-muted gap-1">
                    <Icon class="size-6" name={s().icon} />
                    <Text class="shrink" role="rowheader">{s().label}</Text>
                  </View>
                  <View class="flex-row items-center ml-auto gap-1">
                    <Text class="tabular-nums text-fg w-11 font-bold text-end whitespace-nowrap shrink" role="cell">
                      {s().value}
                    </Text>
                    <View class="flex-row items-center w-16 gap-2 shrink" role="cell">
                      <Show when={s().delta}>
                        <View class={deltaChip} data-trend-down={(s().trend) === "down"} data-trend-up={(s().trend) === "up"}>
                          <Icon name="play" class="hidden bg-current size-2 group-data-trend-up/delta:flex -rotate-90 group-data-trend-down/delta:flex group-data-trend-down/delta:rotate-90" />
                          <Text class="whitespace-nowrap shrink">{s().delta}</Text>
                        </View>
                      </Show>
                    </View>
                    <Text class="tabular-nums text-fg-muted w-12 text-end whitespace-nowrap compact:hidden shrink" role="cell">
                      {s().current}
                    </Text>
                  </View>
                </View>
              )}
            </For>
          </View>
        </Show>
        <Show when={props.lore}>
          <View class="pl-2 border-l-regular border-border-strong gap-2 compact:hidden group-data-tier-uncommon/card:border-uncommon group-data-tier-rare/card:border-rare group-data-tier-epic/card:border-epic group-data-tier-legendary/card:border-legendary group-data-tier-mythic/card:border-mythic">
            <Text class="text-caption text-fg-muted italic m-0">
              {props.lore}
            </Text>
          </View>
        </Show>
      </View>
      <Show when={props.primary || props.secondary}>
        <View class="w-full pt-2 border-t border-border-muted gap-1">
          <Show when={props.primary && !props.canPrimary && props.reason}>
            <Text class="text-danger m-0" role="status">{props.reason}</Text>
          </Show>
          <View class="flex-row items-center w-full gap-1">
            <Show when={props.primary}>
              <Button variant="primary" class={`grow-[1.4] shrink basis-0 min-h-12.75 gap-1 compact:min-h-11`} disabled={!props.canPrimary} label={props.primary} onPress={(e) => props.onEquip?.(e)}>
                <Text glyph="details" class={keyCap} />
                <Text>{props.primary}</Text>
              </Button>
            </Show>
            <Show when={props.secondary}>
              <Button class={`grow shrink basis-0 min-h-12.75 gap-1 compact:min-h-11`} disabled={!props.canSecondary} onPress={(e) => props.onDiscard?.(e)}>
                <Text>{props.secondary}</Text>
              </Button>
            </Show>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgItemCardProps>> = {
  default: {"name":"Emberwake Blade","icon":"sword","art":0,"rarity":"legendary","kind":"Legendary · Longsword","lead":[{"id":"dmg","icon":"sword","label":"Damage","value":"12–20","delta":"+4","trend":"up","current":"8–16"}],"stats":[{"id":"crit","icon":"star","label":"Critical chance","value":"12%","delta":"+4%","trend":"up","current":"8%"},{"id":"res","icon":"shield","label":"Spell resistance","value":"+10%","delta":"−5%","trend":"down","current":"+15%"}],"line":"One-handed · Item level 60","lore":"Forged in a dying star.","primary":"Equip","secondary":"Drop","canPrimary":true,"canSecondary":true},
  plain: {"name":"Bread","icon":"apple","art":0,"rarity":"common","kind":"Common · Food","lead":[],"stats":[],"lore":"","primary":"Use","secondary":"","canPrimary":true,"canSecondary":false},
  refused: {"name":"Staff of the Drowned Choir and the Last Hymn","icon":"drop","art":0,"rarity":"epic","kind":"Epic · Staff","line":"Two-handed","lead":[{"id":"sp","icon":"star","label":"Spell power","value":"38","delta":"−6","trend":"down","current":"44"}],"stats":[{"id":"int","icon":"star","label":"Intellect","value":"+18","delta":"+2","trend":"up","current":"+16"}],"lore":"","primary":"Equip","secondary":"Drop","canPrimary":false,"canSecondary":true,"reason":"Requires level 40"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"details":"Equip"} as const;
