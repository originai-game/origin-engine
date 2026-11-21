import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import "./puzzle.en.ogt";

export interface PuzzleBoostersProps {
  /** The boosters in tray order, keyed rows of { id, icon, name, count, available }: count how many the player holds, as the game counts them; available false when the game will not take it now (none left, not allowed on this level, a move in progress): the booster is disabled, sends nothing and says so. */
  boosters?: ReadonlyArray<{ id: string; icon: string; name: string; count: number; available: boolean }>;
  /** The booster the game has armed (waiting for its target on the board); empty is none. Shown pressed and worded Armed. */
  armed: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * The booster tray: each booster a round socket with its art, how many the
 * player holds on a tag at its foot and its name under it. The one the
 * game has armed is lifted, ringed and tagged Armed; one the game will not
 * take is disabled, its art dimmed, tagged in words (Unavailable), and one
 * the player has none of shows a gold + where its count was. Reports
 * choose (a booster's key; tap, click, Enter or pad A); the game decides
 * whether it applies, spends it and arms or fires it. Many boosters scroll
 * across the tray; a narrow tray wraps them onto more rows.
 */
export function PuzzleBoosters(props: PuzzleBoostersProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-plate-hi to-plate-lo text-fg p-1 gap-2 rounded-block border disabled:text-fg-subtle`} label={t("puzzle.boosters.name")}>
      <Show when={(props.boosters ?? []).length} fallback={
          <Text class={`grow shrink basis-0 text-center text-shadow-2xs p-4 m-0`} role="status">
            {t("puzzle.boosters.empty")}
          </Text>
      }>
        <Scroll class="grow shrink basis-0">
          <View class="flex-row items-center w-max min-w-full justify-center pt-1 pb-0.5 px-0.5 gap-1 compact:gap-y-1 compact:gap-x-0.5 compact:w-full compact:min-w-0 compact:justify-center compact:flex-wrap" role="toolbar" label={t("puzzle.boosters.name")}>
            <For each={props.boosters}>
              {(b) => (
              <View class={`flex-row items-center justify-center gap-1 group/booster w-max min-w-24 max-w-40 min-h-11 py-0.5 px-1 compact:min-w-22.75 compact:max-w-none`} role="button" pressed={b().id === props.armed} disabled={!b().available} label={!b().available ? t("puzzle.boosters.unavailable", { name: b().name }) : b().id === props.armed ? t("puzzle.boosters.armedName", { name: b().name, count: b().count }) : t("puzzle.boosters.count", { name: b().name, count: b().count })} onPress={(e) => props.onChoose?.(e, b().id)} data-armed={b().available && b().id === props.armed} data-not-available={!(b().available)} data-out={b().count === 0}>
                <View class="items-center gap-0.5">
                  <Show when={!b().available || b().id === props.armed} fallback={
                      <View class="h-4.25 mb-0.5" />
                  }>
                    <Text class={`h-4.25 mb-0.5 whitespace-nowrap leading-normal text-xs font-semibold tracking-label uppercase text-fg-muted bg-tile shadow-[inset_0px_1px_0px_#ffffff38] text-shadow-2xs py-0 px-0.5 group-data-armed/booster:text-hud-accent-fg group-data-armed/booster:bg-none group-data-armed/booster:bg-hud-accent`} data-armed={b().available && b().id === props.armed}>
                      {b().available ? t("puzzle.boosters.armed") : t("puzzle.boosters.locked")}
                    </Text>
                  </Show>
                  <View class="size-16 compact:size-12">
                    <View class={`grow shrink basis-0 justify-center items-center absolute shadow-[inset_0px_0px_0px_1.5px_#ffffff47,0px_4.576px_11.424px_#00000080] inset-0 gap-2 rounded-full group-data-armed/booster:shadow-[inset_0px_0px_0px_2.5px_#38e8ff,0px_0px_20.576px_#38e8ff99] group-data-armed/booster:translate-x-0 group-data-armed/booster:-translate-y-1.5`}>
                      <Icon class="text-shadow-2xs size-7/10" name={b().icon} />
                    </View>
                    <Text class={`absolute -right-1 -bottom-0.5 min-w-4 text-center whitespace-nowrap text-xs font-bold text-fg bg-tile nine-chip-cut text-shadow-2xs py-0 px-0.5 data-out:text-gold-fg data-out:bg-none data-out:bg-gold ${packClass("Chip", "frame")}`} data-out={b().count === 0}>
                      {b().count === 0 ? '+' : t("puzzle.boosters.times", { count: formatNumber(b().count) })}
                    </Text>
                  </View>
                  <Text class={`text-fg max-w-32 text-center line-clamp-2 text-xs font-semibold overflow-hidden group-data-armed/booster:text-hud-accent group-data-not-available/booster:text-fg-muted compact:max-w-22.75`}>
                    {b().name}
                  </Text>
                </View>
              </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<PuzzleBoostersProps>> = {
  default: {"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true},{"id":"burst","icon":"fire","name":"Colour burst","count":0,"available":false}]},
  selected: {"armed":"hammer","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true}]},
  disabled: {"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":false},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":0,"available":false},{"id":"moves","icon":"plus","name":"Extra moves","count":2,"available":false}]},
  empty: {"armed":"","boosters":[]},
  overflow: {"armed":"b4","boosters":[{"id":"b1","icon":"bomb","name":"Hammer","count":999,"available":true},{"id":"b2","icon":"bolt","name":"Shuffle","count":12,"available":true},{"id":"b3","icon":"plus","name":"Extra moves","count":1250,"available":true},{"id":"b4","icon":"fire","name":"Colour burst of the Whispering Lemonade Falls","count":7,"available":true},{"id":"b5","icon":"star","name":"Lucky star","count":0,"available":false},{"id":"b6","icon":"key","name":"Unlock","count":2,"available":true},{"id":"b7","icon":"gem","name":"Gem rain","count":40,"available":false},{"id":"b8","icon":"drop","name":"Jelly spill","count":3,"available":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Use"} as const;
