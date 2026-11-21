import {For, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./card.en.ogt";

export interface CardDeckListProps {
  /** The deck's name as the player named it. */
  title: string;
  /** The deck's cards in the game's order (by cost, then name), keyed rows of { id, name, cost, type, rarity, copies, canAdd, canRemove }: copies the game's count; canAdd and canRemove the game's answers (its copy limit, its deck limit, a card the deck must keep). */
  entries: ReadonlyArray<{ id: string; name: string; cost: string; type: string; rarity: string; copies: number; canAdd: boolean; canRemove: boolean }>;
  /** How many cards the deck holds (the game's count). */
  total: number;
  /** The size the game requires or allows; 0 hides the size line. total above limit reaches the size as data-over, below as data-under. */
  limit: number;
  /** The entry shown in detail; empty is none. */
  selected: string;
  /** The main button's text (Save deck, Done); empty hides it. */
  confirmLabel: string;
  /** The main button is enabled (the game's verdict on the deck). */
  canConfirm?: boolean;
  /** The add event, with the row's key where it comes from a row. */
  onAdd?: (e: UiEvent, key?: string) => void;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The confirm event, with the row's key where it comes from a row. */
  onConfirm?: (e: UiEvent, key?: string) => void;
  /** The remove event, with the row's key where it comes from a row. */
  onRemove?: (e: UiEvent, key?: string) => void;
}

/**
 * The deck being built: its name, the size against the game's limit (a
 * meter and the figures, data-over and data-under as well as the ink),
 * one row per card with its cost, name and copies, a minus and a plus per
 * row, and the main button. The game counts, sorts and rules; the list
 * reports choose (a row's key), add and remove (a row's plus or minus,
 * its key; X or pad Y anywhere in the focused row is its minus, so it is
 * silent where the game refuses the removal) and confirm. The rows fill
 * the height given and scroll inside it; the size and the button keep
 * the foot.
 */
export function CardDeckList(props: CardDeckListProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg p-4 gap-2 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title}>
      <View class={`flex-row items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-2`}>
        <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg whitespace-nowrap text-ellipsis m-0 overflow-hidden shrink`}>
          {props.title}
        </Text>
      </View>
      <Show when={props.limit}>
        <View class={`group/size grid grid-cols-[1fr_auto] w-full items-baseline py-0 px-1`} data-under={props.total < props.limit} data-over={props.total > props.limit}>
          <Text class="text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-muted">
            {t("card.deck.size")}
          </Text>
          <Text class="whitespace-nowrap text-[18.24px] font-bold text-fg group-data-over/size:text-danger">
            {t("card.deck.count", { count: props.total, limit: props.limit })}
          </Text>
          <View class="col-span-full h-2 bg-track shadow-sunk overflow-hidden" role="meter" label={t("card.deck.size")} valueText={t("card.deck.count", { count: props.total, limit: props.limit })}>
            <View class="absolute left-0 bg-hud-accent inset-y-0 data-tone-danger:text-danger data-tone-danger:bg-none data-tone-danger:bg-danger" data-tone-danger={((props.total > props.limit ? 'danger' : '') === 'danger') || ((props.total > props.limit ? 'danger' : '') === "danger")} style={{ width: `${(100 * (props.limit ? (props.total > props.limit ? 1 : props.total / props.limit) : 0))}%` }} />
          </View>
        </View>
      </Show>
      <Show when={props.entries.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Text class="m-0">{t("card.deck.empty")}</Text>
          </View>
      }>
        <Scroll class="grow shrink basis-0 gap-2">
          <View class="w-full gap-0.5" role="list" label={props.title}>
            <For each={props.entries}>
              {(e) => (
              <View class={`flex-row items-center bg-linear-to-r from-entry-hi to-entry-lo text-rim-strong transition duration-317 w-full pr-0.5 gap-0.5 rounded-control data-selected:bg-transparent data-selected:bg-hud-accent/24 data-type-attack:text-danger data-type-skill:text-info data-type-power:text-epic data-type-curse:text-fg-subtle data-down:opacity-45`} role="listitem" current={e().id === props.selected} data-type-attack={(e().type) === 'attack'} data-type-curse={(e().type) === 'curse'} data-type-power={(e().type) === 'power'} data-type-skill={(e().type) === 'skill'} data-type-status={(e().type) === 'status'} data-selected={e().id === props.selected}>
                <Button class={`grow shrink basis-0 min-h-11 px-2 text-fg ${packClass("Button", "plate")}`} label={e().name} onPress={(ev) => props.onChoose?.(ev, e().id)}>
                  <View class="flex-row items-center w-full gap-2">
                    <View class={`flex-row items-center w-[1.821rem] h-[2.036rem] justify-center bg-linear-to-b from-hud-accent-hi to-hud-accent text-hud-accent-fg font-bold gap-0`} label={t("card.face.cost", { cost: e().cost })}>
                      <Text class="shrink">{e().cost}</Text>
                    </View>
                    <Text class={`grow shrink basis-0 text-start whitespace-nowrap text-ellipsis font-semibold overflow-hidden`}>
                      {e().name}
                    </Text>
                    <Text class={`whitespace-nowrap font-bold`}>
                      {t("card.deck.copies", { count: e().copies })}
                    </Text>
                  </View>
                </Button>
                <View class={`flex-row gap-1 w-11 min-h-11 items-center justify-center bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg p-0 disabled:text-fg-muted`} role="button" disabled={!e().canRemove} label={t("card.deck.remove", { name: e().name })} onPress={(ev) => props.onRemove?.(ev, e().id)} onAction={{ details: (ev) => props.onRemove?.(ev, e().id) }}>
                  <Text class="text-center">{t("card.deck.less")}</Text>
                </View>
                <View class={`flex-row gap-1 w-11 min-h-11 items-center justify-center bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg p-0 disabled:text-fg-muted`} role="button" disabled={!e().canAdd} label={t("card.deck.add", { name: e().name })} onPress={(ev) => props.onAdd?.(ev, e().id)}>
                  <Text class="text-center">{t("card.deck.more")}</Text>
                </View>
              </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
      <Show when={props.confirmLabel}>
        <View class="flex-row items-center w-full gap-2">
          <Button class={`text-accent w-full min-h-11 shrink ${packClass("Button", "plate", "primary")}`} disabled={!props.canConfirm} onPress={(e) => props.onConfirm?.(e)}>
            <Text>{props.confirmLabel}</Text>
          </Button>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardDeckListProps>> = {
  default: {"title":"Ember Deck","total":12,"limit":30,"selected":"strike","confirmLabel":"Save deck","canConfirm":false,"entries":[{"id":"quick","name":"Quickstep","cost":"0","type":"skill","rarity":"uncommon","copies":2,"canAdd":true,"canRemove":true},{"id":"strike","name":"Ember Strike","cost":"1","type":"attack","rarity":"common","copies":3,"canAdd":false,"canRemove":true},{"id":"guard","name":"Iron Guard","cost":"1","type":"skill","rarity":"common","copies":2,"canAdd":true,"canRemove":true},{"id":"surge","name":"Arcane Surge","cost":"3","type":"power","rarity":"rare","copies":1,"canAdd":false,"canRemove":true},{"id":"oath","name":"Oath of Embers","cost":"2","type":"power","rarity":"epic","copies":1,"canAdd":false,"canRemove":false}]},
  empty: {"title":"New Deck","total":0,"limit":30,"selected":"","confirmLabel":"Save deck","canConfirm":false,"entries":[]},
  full: {"title":"Ember Deck","total":31,"limit":30,"selected":"","confirmLabel":"Save deck","canConfirm":false,"entries":[{"id":"strike","name":"Ember Strike","cost":"1","type":"attack","rarity":"common","copies":3,"canAdd":false,"canRemove":true},{"id":"lament","name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","type":"power","rarity":"legendary","copies":1,"canAdd":false,"canRemove":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"details":"Remove one"} as const;
