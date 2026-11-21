import type { Texture } from "@origingame/engine";
import {For, Scroll, Show, Text, packClass, type UiEvent, View, formatCompact, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import { Counter } from "../controls/counter";
import { Progress } from "../controls/progress";
import "./results.en.ogt";

export interface ResultsScreenProps {
  /** The player won. */
  won?: boolean;
  /** The grade (S, A, B …; empty: none). */
  grade: string;
  /** The result's line (Victory, Stage clear). */
  title: string;
  /** The stats: { name, value, best (bool), format (number or percent; defaults to number), count (bool), decimals }. A stat with count counts up from 0 as the screen opens, written whole or with decimals digits after the point; the others show their value as it is, fractions kept. */
  stats: ReadonlyArray<{ best: boolean; count?: boolean; id: string; name: string; value: number; format?: string; decimals?: number }>;
  /** The level after the match. */
  level: number;
  /** Progress to the next level after the match, 0 to 1. */
  xp: number;
  /** XP gained. */
  gained: number;
  /** The rewards: { name, amount, texture }. */
  rewards: ReadonlyArray<{ amount: number; id: string; name: string; texture: Texture | 0 }>;
  /** summary, graphs or board. */
  tab: string;
  /** Content the caller supplies (was the celebration snippet). */
  celebration?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the graph snippet). */
  graph?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the scores snippet). */
  scores?: (() => JSX.Element) | undefined;
  /** The again event, with the row's key where it comes from a row. */
  onAgain?: (e: UiEvent, key?: string) => void;
  /** The continue event, with the row's key where it comes from a row. */
  onContinue?: (e: UiEvent, key?: string) => void;
  /** The replay event, with the row's key where it comes from a row. */
  onReplay?: (e: UiEvent, key?: string) => void;
  /** The tab switched (onTab), the tab's value (summary, graphs or board). */
  onTab?: (e: UiEvent, value: string) => void;
}

/**
 * After a match: the verdict set large in a lit fill (gold for a win, red
 * for a loss) over the dimmed scene, a rule with a lit diamond, the level
 * in an octagon beside the XP gained and the leaning progress bar, the
 * grade on a hexagon, then the tabs. Summary shows each stat on a cut
 * plate (a best in gold) and each reward as octagon art with its amount
 * on a tag and its name under it. The page scrolls over the actions.
 */
export function ResultsScreen(props: ResultsScreenProps) {
  return (
    <View class="bg-linear-to-r from-black/90 via-black/62 via-55% to-black/81 py-8 px-16 compact:p-3 short:py-3 short:px-4">
      <View class={`text-fg max-w-384 p-0 mx-auto gap-4 size-full disabled:text-fg-subtle short:flex-row compact:gap-3 shrink ${packClass("Panel", "frame")}`}>
        <Scroll class="grow shrink basis-0 gap-6 compact:justify-start compact:gap-3 short:justify-start short:gap-2">
          <View class="items-end gap-6 compact:items-start compact:gap-3">
            <View class="grow shrink basis-0 gap-3 compact:gap-2">
              <Text class={`font-display font-bold leading-[0.9] tracking-xs uppercase text-transparent bg-linear-to-b from-white from-18% via-[#ffecaa] via-52% to-warn m-0 short:text-[4.429rem] data-outcome-lost:bg-linear-to-b data-outcome-lost:from-white data-outcome-lost:from-10% data-outcome-lost:via-danger data-outcome-lost:via-48% data-outcome-lost:to-[#de2634] data-outcome-lost:bg-transparent`} data-outcome-lost={!props.won} data-outcome-won={props.won}>
                {props.title}
              </Text>
              <View class="h-[1.5px]">
                <Text class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] rotate-45 bg-white shadow-[0px_0px_11.424px_#38e8ff] size-[0.643rem]" />
              </View>
              <View class="items-center max-w-[45.714rem] gap-3">
                <Text class="text-center leading-[3.857rem] bg-linear-to-b from-[#463482] to-[#1c143b] text-fg text-[1.714rem] font-bold tabular-nums size-[3.857rem] compact:leading-[2.857rem] compact:text-[1.143rem] compact:size-[2.857rem]">
                  {props.level}
                </Text>
                <View class="grow shrink basis-0 gap-1">
                  <View class="flex-row flex-wrap items-baseline justify-between gap-x-3">
                    <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 m-0 compact:hidden shrink">
                      {t("screen.results.LevelValue", { level: props.level })}
                    </Text>
                    <Text class="text-[1.143rem] font-bold tabular-nums text-epic m-0 shrink">
                      {t("screen.results.Gained", { gained: props.gained })}
                    </Text>
                  </View>
                  <Progress class="w-full h-[0.714rem]" value={props.xp} />
                </View>
              </View>
            </View>
            <Show when={props.grade !== ""}>
              <View class="items-center gap-1">
                <View class="bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 p-0.75">
                  <Text class={`text-center leading-flat font-bold text-fg m-0 compact:leading-[6.143rem] compact:w-[5.357rem] compact:h-[6.143rem] compact:text-[3.571rem] short:leading-[6.143rem] short:w-[5.357rem] short:h-[6.143rem] short:text-[3.571rem]`} role="heading">
                    {props.grade}
                  </Text>
                </View>
                <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 m-0 compact:hidden">
                  {t("screen.results.grade")}
                </Text>
              </View>
            </Show>
          </View>
          <View>{props.celebration?.()}</View>
          <View class="flex-row w-full" role="tablist" label={t("screen.results.grade")}>
            <For each={[{ value: "summary", label: "Summary" }, { value: "graphs", label: "Graphs" }, { value: "board", label: "Scoreboard" }] as const}>
              {(tab) => (
                <View class={`justify-center items-center grow min-h-11 border-b-2 border-transparent data-selected:border-accent data-selected:text-accent text-fg-muted`} role="tab" tabIndex={0} selected={props.tab === tab().value} label={tab().label} onPress={(e) => props.onTab?.(e, tab().value)}>
                  <Text>{tab().label}</Text>
                </View>
              )}
            </For>
          </View>
          <Show when={props.tab === "summary"} fallback={
              <Show when={props.tab === "graphs"} fallback={
                  <Show when={props.tab === "board"}>
                    <View>{props.scores?.()}</View>
                  </Show>
              }>
                <View>{props.graph?.()}</View>
              </Show>
          }>
            <View class="grid grid-cols-[minmax(0,1.7fr)_minmax(0,1fr)] items-start gap-8 compact:grid-cols-[minmax(0,1fr)] compact:gap-4 short:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
              <View class="grid grid-cols-[repeat(auto-fit,minmax(194.3px,1fr))] gap-3 compact:grid-cols-[minmax(0,1fr)] compact:gap-1 short:grid-cols-[minmax(0,1fr)] short:gap-1">
                <For each={props.stats}>
                  {(s) => (
                  <View class={`nine-plate-cut bg-plate-hi text-fg transition duration-317 border border-white/18 justify-between py-3 px-4 gap-2 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 compact:flex-row compact:min-h-0 compact:items-center compact:py-2 compact:px-3 short:flex-row short:min-h-0 short:items-center short:py-1 short:px-3`} data-best={s().best}>
                    <View class="flex-row flex-wrap items-center justify-between gap-1 compact:justify-start compact:grow compact:shrink compact:basis-0 short:justify-start short:grow short:shrink short:basis-0">
                      <Text class={`tracking-[0.14em] uppercase text-fg/78 text-[0.857rem] font-semibold m-0 shrink`}>
                        {s().name}
                      </Text>
                      <Show when={s().best}>
                        <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase text-[#141a02] bg-[#fff2c4] nine-chip-cut whitespace-nowrap py-0 px-1 ${packClass("Chip", "frame")}`}>
                          {t("screen.results.best")}
                        </Text>
                      </Show>
                    </View>
                    <Show when={s().count} fallback={
                        <Text class={`tabular-nums text-fg leading-flat font-bold whitespace-nowrap m-0 compact:text-[2.143rem] short:text-[2.143rem]`}>
                          {t("screen.results.StatValue", { format: s().format, value: s().format === 'percent' ? s().value * 100 : s().value })}
                        </Text>
                    }>
                      <Counter class={`text-fg leading-flat font-bold compact:text-[2.143rem] short:text-[2.143rem]`} value={s().value} from="0" format={s().format === 'percent' ? 'percent' : 'grouped'} decimals={s().decimals} />
                    </Show>
                  </View>
                  )}
                </For>
              </View>
              <Show when={props.rewards.length}>
                <View class="gap-2">
                  <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 m-0">
                    {t("screen.results.rewards")}
                  </Text>
                  <View class="flex-row flex-wrap gap-3">
                    <For each={props.rewards}>
                      {(r) => (
                      <View class={`text-fg transition duration-317 items-center w-[8.286rem] rounded-button data-selected:border-accent data-done:border-success data-active:border-accent disabled:text-fg-subtle disabled:scale-100 active:scale-96 data-locked:scale-100 data-down:opacity-45 compact:w-24 shrink`}>
                        <View class="bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 p-0.75 gap-2 size-[7.429rem] compact:size-[4.857rem]">
                          <Art class="absolute bg-linear-to-b from-plate-hi to-plate-lo object-cover inset-0" kind="itemIcon" src={r().texture} />
                        </View>
                        <Text class="-mt-3 mb-0 text-[0.857rem] font-bold tabular-nums text-fg bg-tile shadow-[inset_0px_1px_0px_#ffffff38] whitespace-nowrap py-0 px-1 mx-0">
                          {t("screen.results.Amount", { r_amount: formatCompact(r().amount) })}
                        </Text>
                        <Text class="mt-1 mb-0 w-full text-center text-[0.929rem] font-semibold text-fg mx-0">
                          {r().name}
                        </Text>
                      </View>
                      )}
                    </For>
                  </View>
                </View>
              </Show>
            </View>
          </Show>
        </Scroll>
        <View class="flex-row flex-wrap justify-end gap-2 compact:grid compact:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] short:flex-nowrap short:justify-end short:flex-col">
          <Button class={`min-h-11 min-w-36 compact:min-w-0 compact:col-span-full compact:row-[2] shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onReplay?.(e)}>
            <Text>{t("screen.results.replay")}</Text>
          </Button>
          <Button class={`min-h-11 min-w-36 compact:min-w-0 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onAgain?.(e)}>
            <Text>{t("screen.results.again")}</Text>
          </Button>
          <Button class={`text-accent min-h-11 min-w-36 compact:min-w-0 shrink ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onContinue?.(e)} onAction={{ confirm: (e) => props.onContinue?.(e) }}>
            <Text>{t("screen.results.continue")}</Text>
          </Button>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ResultsScreenProps>> = {
  default: {"gained":1240,"grade":"A","level":18,"rewards":[{"amount":320,"id":"item-0","name":"Gold","texture":0},{"amount":1,"id":"item-1","name":"Crate","texture":0}],"stats":[{"best":true,"count":true,"id":"item-0","name":"Score","value":48210},{"best":false,"id":"item-1","name":"Kills","value":14},{"best":false,"format":"percent","id":"item-2","name":"Accuracy","value":0.62}],"tab":"summary","title":"Victory","won":true,"xp":0.45},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"results","layer":"menu","focus":"Panel/Footer/Continue","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
