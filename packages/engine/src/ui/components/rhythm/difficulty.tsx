import { For, Icon, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmDifficultyProps {
  /** The heading (the song's title); empty takes the kit's (Difficulty). */
  title: string;
  /** The song's charts in the game's order, keyed rows of { id, tier, label, rating, grade, score, locked }: tier the chart's family (easy, normal, hard, expert, master, special; on data-tier for the pack's colour, never alone); label its name as the game words it (Hard); rating its level number; grade the player's best grade on it (S, AA; empty unplayed); score the best score as the game formats it (981,200; empty Not played); locked it cannot be played yet (disabled). */
  levels: ReadonlyArray<{ id: string; tier: string; label: string; rating: number; grade: string; score: string; locked: boolean }>;
  /** The chart the game holds as chosen (selected, named Selected); empty is none. */
  selected: string;
  /** Choose the chart; the tile's key. A locked chart is disabled and reports nothing. */
  onChoose?: (e: UiEvent, key: string) => void;
}

/**
 * Choosing a chart: one leaning tile per difficulty with its tier's colour
 * along its top, the tier's name, its level as a large figure, the
 * player's best grade on a tag beside the best score, and Locked or
 * Selected in words. A tile reports choose with its key; a locked chart is
 * disabled and reports nothing. The game owns the charts, unlocking,
 * grades and scores.
 */
export function RhythmDifficulty(props: RhythmDifficultyProps) {
  return (
    <View class={`flex-col gap-1 w-full min-w-0 text-fg`} label={props.title !== "" ? props.title : t("rhythm.difficulty.title")}>
      <View class={`flex-row shrink w-full min-h-0`}>
        <Text class={`text-xs font-semibold tracking-label uppercase min-w-0 whitespace-nowrap text-fg text-stroke-3 text-stroke-bg overflow-hidden`}>
          {props.title !== "" ? props.title : t("rhythm.difficulty.title")}
        </Text>
      </View>
      <Show when={(props.levels ?? []).length} fallback={
        <View role="status" class={`self-start py-2 px-3 bg-well`}>
          <Text class={`text-base text-fg`}>{t("rhythm.difficulty.empty")}</Text>
        </View>
      }>
        <Scroll class={`min-h-0 flex-1`}>
          <View role="listbox" label={t("rhythm.difficulty.list")} class={`flex flex-row flex-wrap gap-2 py-0.5 px-0.5 pt-2 w-full`}>
            <For each={props.levels ?? []} key={(l) => l.id}>
              {(l) => (
                <View
                  class={`group/difficulty relative grow shrink basis-37.5 min-w-0 min-h-22 compact:min-h-11 flex-col items-stretch justify-start pt-3.25 compact:pt-1.75 px-4.5 compact:px-2.5 pb-2 compact:pb-1 transition duration-control bg-tile hover:bg-glass shadow-none rounded-none data-selected:bg-surface-hover data-selected:inset-shadow-bar data-selected:inset-shadow-hud-accent data-selected:-translate-y-1.5 compact:data-selected:translate-y-0 disabled:opacity-dim disabled:bg-tile disabled:shadow-none`}
                  role="option"
                  label={t("rhythm.difficulty.ratingName", { label: l().label, rating: l().rating })}
                  selected={l().id === props.selected}
                  disabled={l().locked}
                  onPress={(e) => props.onChoose?.(e, l().id)}
                  data-locked={l().locked}
                  data-tier-easy={l().tier === "easy"}
                  data-tier-normal={l().tier === "normal"}
                  data-tier-hard={l().tier === "hard"}
                  data-tier-expert={l().tier === "expert"}
                  data-tier-master={l().tier === "master"}
                  data-tier-special={l().tier === "special"}
                  data-selected={l().id === props.selected}
                >
                  <View class={`absolute left-0 right-0 top-0 h-1.25 bg-chart-normal group-data-tier-easy/difficulty:bg-chart-easy group-data-tier-hard/difficulty:bg-chart-hard group-data-tier-expert/difficulty:bg-chart-expert group-data-tier-master/difficulty:bg-chart-master group-data-tier-special/difficulty:bg-fever`} />
                  <View class={`flex-col items-start grow gap-0.5 compact:flex-row compact:flex-wrap compact:items-center compact:gap-x-1`}>
                    <View class={`flex-row items-center flex-wrap gap-y-0.5 gap-x-1 max-w-full min-w-0 compact:flex-1 compact:basis-full`}>
                      <Text class={`text-xs font-bold tracking-label uppercase min-w-0 max-w-full whitespace-nowrap text-ellipsis overflow-hidden text-chart-normal group-data-tier-easy/difficulty:text-chart-easy group-data-tier-hard/difficulty:text-chart-hard group-data-tier-expert/difficulty:text-chart-expert group-data-tier-master/difficulty:text-chart-master group-data-tier-special/difficulty:text-fever-ink group-data-selected/difficulty:text-fg`}>
                        {l().label}
                      </Text>
                      <Show when={l().locked}>
                        <View class={`flex-row items-center gap-1 shrink py-0 px-1 ${packClass("Chip", "frame")}`} data-tag-locked={true}>
                          <Icon class={`size-4 opacity-quiet`} name="lock" />
                          <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap opacity-quiet`}>{t("rhythm.difficulty.locked")}</Text>
                        </View>
                      </Show>
                      <Show when={l().id === props.selected}>
                        <View class={`flex-row items-center shrink py-0 px-1 ${packClass("Chip", "frame")}`} data-tag-selected={true}>
                          <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.difficulty.selected")}</Text>
                        </View>
                      </Show>
                    </View>
                    <View class={`self-start shrink compact:self-center`}>
                      <Text class={`text-7xl compact:text-3xl leading-none font-bold tabular-nums whitespace-nowrap text-fg disabled:text-fg-muted`}>
                        {t("rhythm.difficulty.rating", { rating: l().rating })}
                      </Text>
                    </View>
                    <View class={`flex-row items-center max-w-full min-w-0 compact:flex-none`}>
                      <Show when={l().grade !== ""}>
                        <View class={`flex-row items-center shrink py-0 px-1 text-gold ${packClass("Chip", "frame")}`} data-tag-grade={true}>
                          <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{l().grade}</Text>
                        </View>
                      </Show>
                      <Text class={`min-w-0 text-base font-bold tabular-nums whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted`}>
                        {l().score !== "" ? l().score : t("rhythm.difficulty.unplayed")}
                      </Text>
                    </View>
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

export const fixtures: Record<string, Partial<RhythmDifficultyProps>> = {
  default: {"title":"","selected":"hard","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":9,"grade":"","score":"","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true}]},
  selected: {"title":"Skyline Drive","selected":"easy","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false}]},
  disabled: {"title":"","selected":"","levels":[{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true},{"id":"master","tier":"master","label":"Master","rating":14,"grade":"","score":"","locked":true}]},
  empty: {"title":"","selected":"","levels":[]},
  overflow: {"title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","selected":"special","levels":[{"id":"beginner","tier":"easy","label":"Beginner","rating":1,"grade":"SSS+","score":"1,000,000","locked":false},{"id":"easy","tier":"easy","label":"Easy","rating":4,"grade":"SS","score":"998,870","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":7,"grade":"S","score":"975,010","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":10,"grade":"AA","score":"931,445","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":13,"grade":"B","score":"801,992","locked":false},{"id":"master","tier":"master","label":"Master","rating":15,"grade":"","score":"","locked":false},{"id":"special","tier":"special","label":"Remaster Ultimate Challenge","rating":16.5,"grade":"","score":"","locked":false},{"id":"world","tier":"special","label":"Final Ascent","rating":99,"grade":"","score":"","locked":true}]},
};
