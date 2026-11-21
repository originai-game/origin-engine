import type { Texture } from "@origingame/engine";
import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { CardFace } from "./face";
import "./card.en.ogt";

export interface CardRewardProps {
  /** The heading as the game words it (Choose a reward, Treasure). */
  title: string;
  /** A small line over the title naming the moment (Victory · Floor 7); empty hides it. */
  kicker?: string;
  /** The offers the game rolled, keyed rows of { id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }: relic true draws a relic (its icon, typeLabel or Relic, name and text) instead of a card; enabled false when the game will not let it be taken. */
  choices: ReadonlyArray<{ id: string; relic: boolean; name: string; cost: string; icon: string; art: Texture | 0; type: string; typeLabel: string; rarity: string; text: string; enabled: boolean }>;
  /** The offer the player is looking at (focused or tapped once); empty is none. */
  selected: string;
  /** The skip button's text (Skip, Take nothing); empty hides it (Back still reports skip; a game that offers none ignores it). */
  skipLabel: string;
  /** The pick event, with the row's key where it comes from a row. */
  onPick?: (e: UiEvent, key?: string) => void;
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
}

/**
 * Pick one of the game's offers: cards drawn as cards, relics as a gold
 * plate with their mark, kind, name and text, and a skip button. The game
 * rolls the offers and applies the pick; the screen reports pick (an
 * offer's key: tap, Enter, pad A) and skip (the button, Escape, pad B).
 * An offer the game refuses is disabled and dimmed; the one the player is
 * looking at wears brackets. The offers share one row while it fits; a
 * phone shows one at nearly full width, so its text stays whole, with the
 * next one peeking at the faded edge, and scrolls sideways.
 */
export function CardReward(props: CardRewardProps) {
  return (
    <View class={`grow shrink basis-0 text-fg items-center justify-center p-0 gap-4 disabled:text-fg-subtle`} label={props.title} onAction={{ back: (e) => props.onSkip?.(e) }}>
      <View class={`min-h-12 border-b border-rim-soft w-full items-center py-1 px-4 gap-0.5`}>
        <Show when={props.kicker}>
          <Text class={`text-center whitespace-nowrap text-ellipsis text-[0.857rem] font-semibold tracking-[0.18em] uppercase text-fg-muted overflow-hidden`}>
            {props.kicker}
          </Text>
        </Show>
        <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg text-center m-0`}>
          {props.title}
        </Text>
      </View>
      <Scroll class="grow shrink basis-0 w-full">
        <View class="flex-row items-center w-full justify-center p-2 gap-3 compact:w-auto compact:justify-start compact:pr-12" role="listbox" label={t("card.reward.choices")}>
          <Show when={(props.choices)?.length ?? 0} fallback={<Text class="text-center m-0 shrink" role="status">{t("card.reward.empty")}</Text>}><For each={props.choices}>
            {(c) => (
            <Show when={c().relic} fallback={
                <View class={`shrink min-w-[105.6px] aspect-[7/10] justify-stretch p-0 compact:grow-0 compact:shrink-0 compact:min-w-0`} role="option" disabled={!c().enabled} selected={c().id === props.selected} label={c().name} description={c().text} onPress={(e) => props.onPick?.(e, c().id)}>
                  <View class="w-full">
                    <View class="w-full">
                      <CardFace name={c().name} cost={c().cost} icon={c().icon} art={c().art} type={c().type} typeLabel={c().typeLabel} rarity={c().rarity} text={c().text} dim={!c().enabled} />
                    </View>
                    <Show when={c().id === props.selected}>
                      <View class={`absolute pointer-events-none -inset-0.5`} />
                    </Show>
                  </View>
                </View>
            }>
              <View class={`group/relic shrink min-w-[105.6px] aspect-[7/10] justify-stretch p-0 compact:grow-0 compact:shrink-0 compact:min-w-0`} role="option" disabled={!c().enabled} selected={c().id === props.selected} label={c().name} description={c().text} onPress={(e) => props.onPick?.(e, c().id)}>
                <View class="w-full">
                  <View class={`w-full items-center justify-center text-center border-[1.5px] border-gold/70 nine-plate-cut p-2 gap-1 overflow-hidden ${packClass("Plate", "frame")}`}>
                    <View class={`flex-row items-center w-[42%] aspect-square justify-center bg-tile-pressed gap-0`}>
                      <Icon class="text-gold size-[58%] shrink" name={c().icon} />
                    </View>
                    <Text class={`w-full whitespace-nowrap text-ellipsis text-[0.786rem] font-bold tracking-[0.14em] uppercase text-gold overflow-hidden`}>
                      {c().typeLabel || t("card.reward.relic")}
                    </Text>
                    <Text class={`text-fg w-full line-clamp-2 text-base font-bold overflow-hidden`}>
                      {c().name}
                    </Text>
                    <Text class={`w-full line-clamp-5 text-[0.893rem] text-fg overflow-hidden`}>
                      {c().text}
                    </Text>
                  </View>
                  <Show when={c().id === props.selected}>
                    <View class={`absolute pointer-events-none -inset-0.5`} />
                  </Show>
                </View>
              </View>
            </Show>
            )}
          </For></Show>
        </View>
      </Scroll>
      <Show when={props.skipLabel}>
        <View class="flex-row items-center justify-center gap-2">
          <Button class={`min-w-48 min-h-11 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onSkip?.(e)}>
            <Text>{props.skipLabel}</Text>
          </Button>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardRewardProps>> = {
  default: {"title":"Choose a reward","kicker":"Victory · Floor 7","selected":"cleave","skipLabel":"Skip","choices":[{"id":"cleave","relic":false,"name":"Cleave","cost":"1","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage to all enemies.","enabled":true},{"id":"mend","relic":false,"name":"Mend","cost":"2","icon":"heart","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Heal 6. Exhaust.","enabled":true},{"id":"surge","relic":false,"name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","enabled":true}]},
  relics: {"title":"Treasure","kicker":"","selected":"","skipLabel":"Take nothing","choices":[{"id":"lantern","relic":true,"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"","rarity":"rare","text":"At the start of each combat, gain 1 energy.","enabled":true},{"id":"vial","relic":true,"name":"Bottled Storm","cost":"","icon":"drop","art":0,"type":"relic","typeLabel":"","rarity":"epic","text":"Your first attack each turn deals 4 more damage.","enabled":false}]},
  empty: {"title":"Choose a reward","kicker":"","selected":"","skipLabel":"Continue","choices":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Skip"} as const;
