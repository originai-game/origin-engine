import type { Texture } from "@origingame/engine";
import {For, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { CardFace } from "./face";
import "./card.en.ogt";

export interface CardCollectionProps {
  /** The collection's heading (Collection, Library); empty hides the band. */
  title: string;
  /** The filters the game offers (classes, costs, types), keyed rows of { id, label, count } (count above 0 shows beside the label); empty hides the row. */
  filters: ReadonlyArray<{ id: string; label: string; count?: number }>;
  /** The id of the filter applied; the game filters the cards it passes. */
  filter: string;
  /** The cards shown, already filtered and sorted by the game: keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }. owned and inDeck are the game's counts (owned above 1 is shown, inDeck above 0); canAdd is the game's answer (limit, rarity rule, not owned); locked marks a card not owned yet; reason is why the game will not add it, in the game's words (spoken and marked when canAdd is false; empty says Not owned for a locked card, Cannot be added otherwise). */
  cards: ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture | 0; type: string; typeLabel: string; rarity: string; text: string; owned: number; inDeck: number; canAdd: boolean; locked: boolean; reason: string }>;
  /** The card shown in detail; empty is none. */
  selected: string;
  /** What an empty result says; empty is No cards match. */
  empty: string;
  /** The add event, with the row's key where it comes from a row. */
  onAdd?: (e: UiEvent, key?: string) => void;
  /** The choosefilter event, with the row's key where it comes from a row. */
  onChoosefilter?: (e: UiEvent, key?: string) => void;
  /** The inspect event, with the row's key where it comes from a row. */
  onInspect?: (e: UiEvent, key?: string) => void;
  /** The tabNext event, with the row's key where it comes from a row. */
  onTabNext?: (e: UiEvent, key?: string) => void;
  /** The tabPrev event, with the row's key where it comes from a row. */
  onTabPrev?: (e: UiEvent, key?: string) => void;
}

/**
 * The card library of a deck builder: a heading, filter tabs and a grid
 * of cards that fills the height it is given and scrolls inside it. The
 * game filters and sorts what it passes; the grid reports add (a card's
 * key: tap, Enter, pad A), inspect (X or pad Y on the focused card) and
 * choosefilter (a filter's key), tabNext and tabPrev (E/Q, bumpers). A
 * card the game will not add never reports add: it stays in reach, so it
 * can be focused and read, reports inspect from a tap too, and says why
 * in the game's words, with a lock when it is not owned and a cross when
 * it is. The core DataGrid is a table of text rows, so the cards are a
 * grid of their own.
 */
export function CardCollection(props: CardCollectionProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg p-4 gap-2 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("card.collection.cards")}>
      <Show when={props.title}>
        <View class={`flex-row items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-2`}>
          <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0 shrink`}>
            {props.title}
          </Text>
        </View>
      </Show>
      <Show when={props.filters.length}>
        <View class="w-full border-b border-rim-soft" role="tablist" label={t("card.collection.filters")} onAction={{ tabNext: (e) => props.onTabNext?.(e), tabPrev: (e) => props.onTabPrev?.(e) }}>
          <Scroll class="w-full">
            <View class="flex-row items-center pr-12 gap-1">
              <For each={props.filters}>
                {(f) => (
                <View class={`flex-row items-center justify-center group/filter min-h-11 min-w-11 text-fg-muted py-0 px-2 gap-1 data-selected:text-fg`} role="tab" selected={f().id === props.filter} label={f().label} onPress={(e) => props.onChoosefilter?.(e, f().id)} data-selected={f().id === props.filter}>
                  <Text class="whitespace-nowrap">{f().label}</Text>
                  <Show when={(f().count ?? 0) > 0}>
                    <Text class={`text-[0.857rem] font-bold bg-white/8 text-fg-muted py-0 px-1 rounded-capsule group-data-selected/filter:bg-none group-data-selected/filter:bg-hud-accent group-data-selected/filter:text-hud-accent-fg`}>
                      {formatNumber(f().count ?? 0)}
                    </Text>
                  </Show>
                  <View class="absolute bottom-0 h-0.75 bg-hud-accent shadow-[0px_0px_11.424px_#38e8ff] opacity-0 inset-x-1 group-data-selected/filter:opacity-100" />
                </View>
                )}
              </For>
            </View>
          </Scroll>
        </View>
      </Show>
      <Show when={props.cards.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Text class="m-0">{props.empty || t("card.collection.empty")}</Text>
          </View>
      }>
        <Scroll class="gap-2">
          <View class="grid grid-cols-[repeat(auto-fill,minmax(128px,1fr))] w-full p-1" role="grid" label={props.title || t("card.collection.cards")}>
            <For each={props.cards}>
              {(c) => (
              <View class="items-center gap-0.5">
                <Show when={c().canAdd} fallback={
                    <View class={`text-fg transition duration-317 w-full aspect-[7/10] justify-stretch p-0 data-selected:border-accent data-done:border-success data-active:border-accent disabled:text-fg-subtle disabled:scale-100 active:scale-96 data-locked:scale-100 data-down:opacity-45`} role="gridcell" data-can-add={c().canAdd} data-locked={c().locked} selected={c().id === props.selected} label={c().name} description={c().reason || (c().locked ? t("card.collection.locked") : t("card.collection.refused"))} onPress={(e) => props.onInspect?.(e, c().id)} onAction={{ details: (e) => props.onInspect?.(e, c().id) }} data-in-deck={c().inDeck > 0} data-selected={c().id === props.selected}>
                      <View class="w-full">
                        <View class="w-full">
                          <CardFace name={c().name} cost={c().cost} icon={c().icon} art={c().art} type={c().type} typeLabel={c().typeLabel} rarity={c().rarity} text={c().text} dim={c().locked} />
                        </View>
                        <Show when={c().locked}>
                          <View class="absolute pointer-events-none inset-0" />
                        </Show>
                        <Show when={c().owned > 0}>
                          <Text class={`absolute right-0.5 top-0.5 nine-chip-cut bg-tile text-fg text-[0.857rem] font-bold py-0 px-0.5 ${packClass("Chip", "frame")}`}>
                            {t("card.collection.owned", { count: c().owned })}
                          </Text>
                        </Show>
                        <Show when={c().id === props.selected}>
                          <View class="absolute pointer-events-none -inset-0.5" />
                        </Show>
                      </View>
                    </View>
                }>
                  <View class={`text-fg transition duration-317 w-full aspect-[7/10] justify-stretch p-0 data-selected:border-accent data-done:border-success data-active:border-accent disabled:text-fg-subtle disabled:scale-100 active:scale-96 data-locked:scale-100 data-down:opacity-45`} role="gridcell" data-can-add={c().canAdd} data-locked={c().locked} selected={c().id === props.selected} label={c().name} description={c().text} onPress={(e) => props.onAdd?.(e, c().id)} onAction={{ details: (e) => props.onInspect?.(e, c().id) }} data-in-deck={c().inDeck > 0} data-selected={c().id === props.selected}>
                    <View class="w-full">
                      <View class="w-full">
                        <CardFace name={c().name} cost={c().cost} icon={c().icon} art={c().art} type={c().type} typeLabel={c().typeLabel} rarity={c().rarity} text={c().text} dim={false} />
                      </View>
                      <Show when={c().owned > 0}>
                        <Text class={`absolute right-0.5 top-0.5 nine-chip-cut bg-tile text-fg text-[0.857rem] font-bold py-0 px-0.5 ${packClass("Chip", "frame")}`}>
                          {t("card.collection.owned", { count: c().owned })}
                        </Text>
                      </Show>
                      <Show when={c().id === props.selected}>
                        <View class="absolute pointer-events-none -inset-0.5" />
                      </Show>
                    </View>
                  </View>
                </Show>
                <Text class={`max-w-full text-center whitespace-nowrap text-ellipsis text-[0.857rem] font-bold text-fg-muted overflow-hidden data-in-deck:text-hud-accent`} data-in-deck={c().inDeck > 0}>
                  {c().inDeck > 0 ? t("card.collection.inDeck", { count: c().inDeck }) : t("card.collection.notInDeck")}
                </Text>
                <Show when={!c().canAdd}>
                  <Text class={`max-w-full text-center text-[0.857rem] font-bold text-danger`}>
                    {c().reason || (c().locked ? t("card.collection.locked") : t("card.collection.refused"))}
                  </Text>
                </Show>
              </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardCollectionProps>> = {
  default: {"title":"Collection","filter":"all","filters":[{"id":"all","label":"All","count":42},{"id":"attack","label":"Attacks","count":18},{"id":"skill","label":"Skills","count":15},{"id":"power","label":"Powers","count":6}],"selected":"guard","empty":"","cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","owned":3,"inDeck":2,"canAdd":true,"locked":false,"reason":""},{"id":"guard","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","owned":2,"inDeck":2,"canAdd":false,"locked":false,"reason":"Both copies are in the deck"},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","owned":1,"inDeck":0,"canAdd":true,"locked":false,"reason":""},{"id":"crown","name":"Hollow Crown","cost":"2","icon":"crown","art":0,"type":"power","typeLabel":"Power","rarity":"legendary","text":"Your attacks deal double damage.","owned":0,"inDeck":0,"canAdd":false,"locked":true,"reason":""}]},
  empty: {"title":"Collection","filter":"power","filters":[{"id":"all","label":"All"},{"id":"power","label":"Powers"}],"selected":"","empty":"","cards":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"tabPrev":"Previous filter","tabNext":"Next filter","details":"Inspect"} as const;
