import {For, Icon, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import "./card.en.ogt";

export interface CardPilesProps {
  /** The piles in the game's order, keyed rows of { id, kind, label, icon, count, enabled }: kind is draw, discard, exhaust or the game's own word (it reaches the pile as data-kind); count the game's number; enabled false when the game will not open it. */
  piles?: ReadonlyArray<{ id: string; kind: string; label: string; icon: string; count: number; enabled: boolean }>;
  /** What a screen reader calls the group; empty is Piles. */
  label: string;
  /** The open event, with the row's key where it comes from a row. */
  onOpen?: (e: UiEvent, key?: string) => void;
}

/**
 * The draw, discard and exhaust piles as buttons: a stack of card backs
 * (one when it is empty) with the pile's icon on top, the game's count on
 * a tag at its corner and a label under it. A pile at 0 carries data-empty;
 * one the game will not open is disabled. Reports open (a pile's key). The
 * piles sit in a row spread across the width they are given; a page that
 * wants the draw pile in one corner and the discard in the other mounts two.
 */
export function CardPiles(props: CardPilesProps) {
  return (
    <View class={`flex-row items-end text-fg w-full justify-between p-0 gap-2 disabled:text-fg-subtle`} label={props.label || t("card.piles.name")}>
      <For each={props.piles}>
        {(p) => (
        <View class={`flex-row gap-1 group/pile shrink min-w-11 min-h-11 items-center justify-center p-1`} role="button" disabled={!p().enabled} label={t("card.piles.count", { label: p().label, count: p().count })} onPress={(e) => props.onOpen?.(e, p().id)} data-kind-discard={(p().kind) === 'discard'} data-empty={p().count === 0}>
          <View class="items-center gap-0.5">
            <View class={`group/stack w-12 h-[67.2px] mr-1 mt-1`}>
              <Show when={p().count > 1}>
                <View class="absolute shadow-[inset_0px_0px_0px_1.5px_#ffffff38] -translate-x-1 -translate-y-1 inset-0" />
              </Show>
              <Show when={p().count > 0}>
                <View class="absolute shadow-[inset_0px_0px_0px_1.5px_#ffffff38] -translate-x-0.5 -translate-y-0.5 inset-0" />
              </Show>
              <View class="flex-row items-center absolute shadow-[inset_0px_0px_0px_1.5px_#ffffff38] justify-center inset-0 gap-0 group-data-empty/stack:bg-none group-data-empty/stack:bg-transparent group-data-empty/stack:shadow-[inset_0px_0px_0px_1.5px_#ffffff47]">
                <Icon class="text-fg-muted size-[56%] shrink" name={p().icon} />
              </View>
              <Text class={`absolute -right-1 -bottom-0.5 min-w-4 text-center whitespace-nowrap nine-chip-cut bg-fg text-fg-inverse text-[1.143rem] font-bold py-0 px-0.5 group-data-kind-discard/pile:bg-none group-data-kind-discard/pile:bg-danger group-data-empty/stack:bg-none group-data-empty/stack:text-fg-muted ${packClass("Chip", "frame")}`}>
                {formatNumber(p().count)}
              </Text>
            </View>
            <Text class={`tracking-[0.18em] uppercase text-fg-muted max-w-36 whitespace-nowrap text-ellipsis text-center text-[0.857rem] font-semibold overflow-hidden`}>
              {p().label}
            </Text>
          </View>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardPilesProps>> = {
  default: {"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":14,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":3,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":1,"enabled":true}]},
  empty: {"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":0,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":0,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":0,"enabled":false}]},
  overflow: {"label":"","piles":[{"id":"draw","kind":"draw","label":"Remaining in the draw pile","icon":"bag","count":1284,"enabled":true},{"id":"discard","kind":"discard","label":"Discarded this combat","icon":"back","count":312,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhausted for good","icon":"fire","count":96,"enabled":false}]},
};
