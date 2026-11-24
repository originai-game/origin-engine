import type { Texture } from "@origingame/engine";
import {For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { CardFace } from "./face";
import "./card.en.ogt";

export interface CardHandProps {
  /** The hand in the game's order, keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, playable, reason, at }: playable is the game's answer (cost, targets, rules), never the hand's; reason the game's words for an unplayable card (No energy), empty Cannot be played; at is the card's place across a fanned hand from -1 (first) to 1 (last), 0 the middle, which the game derives from the order. */
  cards: ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture | 0; type: string; typeLabel: string; rarity: string; text: string; playable: boolean; at: number; reason?: string }>;
  /** The id of the card the player picked up (it rises and offers Play when playable); empty is none. */
  selected: string;
  /** How far a fanned hand turns its end cards, in degrees (the game's choice, 0 a flat row). */
  fan: number;
  /** What a screen reader calls the hand; empty is Hand. */
  label: string;
  /** The play event, with the row's key where it comes from a row. */
  onPlay?: (e: UiEvent, key?: string) => void;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

/**
 * The player's hand: cards side by side, overlapping as the hand grows so
 * every card keeps a visible edge, turned along a fan when the game asks
 * for one. The selected card rises; when the game says it is playable a
 * Play plate sits on it. A card the game says cannot be played is not
 * disabled (it can still be picked up and read) but carries the
 * unplayable mark and data-playable=false, and never reports play.
 * Reports select (a card's key: tap, Enter, pad A) and play (the Play
 * plate, or X / pad Y on a focused playable card, its key). The game
 * decides legality, cost and targets.
 */
export function CardHand(props: CardHandProps) {
  return (
    <View class={`grow shrink basis-0 text-fg justify-center p-0 gap-0 disabled:text-fg-subtle`} label={props.label || t("card.hand.name")}>
      <Show when={props.cards.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Text class={`m-0 py-2 px-4 text-fg bg-glass nine-plate-cut ${packClass("Plate", "frame")}`}>{t("card.hand.empty")}</Text>
          </View>
      }>
        <View class="flex-row items-end grow shrink basis-0 w-full justify-center pt-14 pb-14 px-8 gap-0" role="listbox" label={props.label || t("card.hand.name")}>
          <For each={props.cards}>
            {(c) => (
            <View class={`relative shrink min-w-0 basis-48 h-68.5 compact:basis-30 compact:h-42.75 origin-[50%_100%] data-tail:shrink-0 data-selected:rotate-0 data-selected:translate-x-0 data-selected:-translate-y-12 data-selected:z-1 hover:translate-x-0 hover:-translate-y-2 transition-transform duration-200 ease-out`} data-playable={c().playable} data-tail={c().at === 1 || props.cards.length === 1} data-selected={c().id === props.selected} style={{ rotate: `${((c().at) * (props.fan))}deg` }}>
              <View class="absolute left-0 inset-y-0 w-48 compact:w-30">
                <Show when={c().playable} fallback={
                    <View class={`flex-row items-stretch gap-1 border-rim-soft bg-surface-raised inset-shadow-lit transition duration-control size-full justify-stretch p-0 rounded-block border data-selected:border-accent data-down:opacity-dim`} role="option" data-playable={c().playable} selected={c().id === props.selected} label={c().name} description={c().reason || t("card.hand.unplayable")} onPress={(e) => props.onSelect?.(e, c().id)}>
                      <View class="size-full">
                        <CardFace name={c().name} cost={c().cost} icon={c().icon} art={c().art} type={c().type} typeLabel={c().typeLabel} rarity={c().rarity} text={c().text} dim={true} />
                      </View>
                      <View class={`flex-row items-center absolute left-9 right-1 top-2 nine-plate-cut bg-foe-fg text-danger text-2xs font-bold tracking-xs py-0.5 px-1 gap-0.5 data-not-cost:left-1`} data-not-cost={!(!!c().cost)}>
                        <Icon class="size-2.75" name="lock" />
                        <Text class="text-start line-clamp-2 leading-tight m-0 overflow-hidden shrink">
                          {c().reason || t("card.hand.unplayable")}
                        </Text>
                      </View>
                    </View>
                }>
                  <View class={`flex-row items-stretch gap-1 border-rim-soft bg-surface-raised inset-shadow-lit transition duration-control size-full justify-stretch p-0 rounded-block border data-selected:border-accent data-down:opacity-dim`} role="option" data-playable={c().playable} selected={c().id === props.selected} label={c().name} description={c().text} onPress={(e) => props.onSelect?.(e, c().id)} onAction={{ details: (e) => props.onPlay?.(e, c().id) }}>
                    <View class="size-full">
                      <CardFace name={c().name} cost={c().cost} icon={c().icon} art={c().art} type={c().type} typeLabel={c().typeLabel} rarity={c().rarity} text={c().text} dim={false} />
                    </View>
                  </View>
                </Show>
                <Show when={c().id === props.selected && c().playable}>
                  <Button variant="primary" class={`absolute left-1/10 top-full mt-0.5 w-4/5 min-h-11`} onPress={(e) => props.onPlay?.(e, c().id)}>
                    <Text>{t("card.hand.play")}</Text>
                  </Button>
                </Show>
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardHandProps>> = {
  default: {"label":"","selected":"strike","fan":6,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage. Apply 1 Burn.","playable":true,"at":-1},{"id":"guard","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","playable":true,"at":-0.33},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"reason":"No energy","at":0.33},{"id":"wound","name":"Wound","cost":"","icon":"skull","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable.","playable":false,"at":1}]},
  empty: {"label":"","selected":"","fan":6,"cards":[]},
  single: {"label":"","selected":"","fan":0,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":true,"at":0}]},
  disabled: {"label":"","selected":"surge","fan":0,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":false,"at":-1},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"at":1}]},
  overflow: {"label":"","selected":"c6","fan":10,"cards":[{"id":"c1","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":true,"at":-1},{"id":"c2","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","playable":true,"at":-0.8},{"id":"c3","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"at":-0.6},{"id":"c4","name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","icon":"skull","art":0,"type":"power","typeLabel":"Power","rarity":"legendary","text":"At the start of your turn every enemy loses 3 Strength and you draw 2 cards.","playable":false,"at":-0.4},{"id":"c5","name":"Quickstep","cost":"0","icon":"leaf","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Draw 1 card.","playable":true,"at":-0.2},{"id":"c6","name":"Cleave","cost":"1","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage to all enemies.","playable":true,"at":0},{"id":"c7","name":"Mend","cost":"2","icon":"heart","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Heal 6.","playable":true,"at":0.2},{"id":"c8","name":"Wound","cost":"","icon":"skull","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable.","playable":false,"at":0.4},{"id":"c9","name":"Bash","cost":"2","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage. Apply 2 Vulnerable.","playable":true,"at":0.6},{"id":"c10","name":"Ward","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 8 Block.","playable":true,"at":0.8},{"id":"c11","name":"Burn","cost":"","icon":"fire","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable. Take 2 damage at the end of your turn.","playable":false,"at":1}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"details":"Play"} as const;
