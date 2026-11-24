import type { Texture } from "@origingame/engine";
import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatCompact, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import { Counter } from "../controls/counter";
import { Progress } from "../controls/progress";
import "./results.en.ogt";

export interface ResultsScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The player won. */
  won?: boolean;
  /** The grade (S, A, B …; empty: none). */
  grade: string;
  /** The result's line (Victory, Stage clear). */
  title: string;
  /** The stats: { name, value, best (bool), format (number or percent; defaults to number), count (bool), decimals }. A stat with count counts up from 0 as the screen opens, written whole or with decimals digits after the point; the others show their value as it is, fractions kept. */
  stats: ReadonlyArray<{ best: boolean; count?: boolean; id: string; name: string; value: number; format?: string; decimals?: number }>;
  /** The level after the match; absent, no level, XP or progress is drawn (a game without levels). */
  level?: number;
  /** Progress to the next level after the match, 0 to 1. */
  xp?: number;
  /** XP gained. */
  gained?: number;
  /** The rewards: { name, amount, texture, icon? }: texture the reward's picture filling its well; 0 shows icon (an atlas icon name, as a Shop price's) on the well's plate. */
  rewards: ReadonlyArray<{ amount: number; id: string; name: string; texture: Texture | 0; icon?: string }>;
  /** summary, graphs or board. */
  tab: string;
  /** Content the caller supplies (was the celebration snippet). */
  celebration?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the graph snippet). */
  graph?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the scores snippet). */
  scores?: (() => JSX.Element) | undefined;
  /** The again event, with the row's key where it comes from a row. */
  onAgain?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The continue event, with the row's key where it comes from a row. */
  onContinue?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The replay event, with the row's key where it comes from a row. */
  onReplay?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The tab switched (onTab), the tab's value (summary, graphs or board). Without it no tab row is drawn, and the page shows `tab`. */
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
    <View
      class={`absolute inset-0 bg-linear-to-r from-glass via-glass via-55% to-glass pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+2rem)] pl-[calc(env(hud-inset-left)+4rem)] pr-[calc(env(hud-inset-right)+4rem)] compact:pt-[calc(env(hud-inset-top)+0.75rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:pl-[calc(env(hud-inset-left)+0.75rem)] compact:pr-[calc(env(hud-inset-right)+0.75rem)] short:py-3 short:px-4 ${packClass("Screen", "frame")} ${props.class ?? ""}`}
      sound={props.won ? "reward" : "error"}
    >
      <View class={`max-w-384 p-0 mx-auto gap-4 size-full disabled:text-fg-subtle short:flex-row compact:gap-3 compact:h-auto compact:max-h-full compact:my-auto shrink ${packClass("Panel", "frame")}`}>
        <Scroll class="grow shrink basis-0 compact:grow-0 compact:basis-auto gap-6 compact:justify-start compact:gap-3 short:justify-start short:gap-2">
          <View class="flex-row items-end gap-6 compact:items-start compact:gap-3">
            <View class="grow shrink basis-0 gap-3 compact:gap-2">
              <Text class={`font-display text-display-lg compact:text-display-md font-bold leading-flat tracking-xs uppercase text-gold-hi m-0 short:text-8xl data-outcome-lost:text-danger`} data-outcome-lost={!props.won} data-outcome-won={props.won}>
                {props.title}
              </Text>
              <View class="h-px w-full max-w-140 bg-linear-to-r from-transparent via-fg-muted to-transparent" />
              <Show when={props.level !== undefined}>
                <View class="flex-row items-center max-w-160 gap-3">
                  <Text class="text-center leading-13.5 rounded-card bg-linear-to-b from-epic to-glass-lo text-fg text-title font-bold tabular-nums size-13.5 compact:leading-10 compact:text-base compact:size-10">
                    {props.level}
                  </Text>
                  <View class="grow shrink basis-0 gap-1">
                    <View class="flex-row flex-wrap items-baseline justify-between gap-x-3">
                      <Text class="text-xs font-semibold tracking-label uppercase text-fg-muted m-0 compact:hidden shrink">
                        {t("screen.results.LevelValue", { level: props.level })}
                      </Text>
                      <Text class="text-base font-bold tabular-nums text-epic m-0 shrink">
                        {t("screen.results.Gained", { gained: props.gained })}
                      </Text>
                    </View>
                    <Progress class="w-full h-2.5" value={(props.xp ?? 0) * 100} />
                  </View>
                </View>
              </Show>
            </View>
            <Show when={props.grade !== ""}>
              <View class="items-center gap-1">
                <View class="rounded-card bg-linear-to-b from-rim-strong via-rim via-45% to-rim-soft p-0.75">
                  <Text class={`text-center leading-flat font-bold text-fg m-0 compact:leading-21.5 compact:w-18.75 compact:h-21.5 compact:text-display-xl short:leading-21.5 short:w-18.75 short:h-21.5 short:text-display-xl`} role="heading">
                    {props.grade}
                  </Text>
                </View>
                <Text class="text-xs font-semibold tracking-label uppercase text-fg-muted m-0 compact:hidden">
                  {t("screen.results.grade")}
                </Text>
              </View>
            </Show>
          </View>
          <View>{props.celebration?.()}</View>
          <Show when={props.onTab}>
            <View class="flex-row w-full" role="tablist" label={t("screen.results.grade")}>
              <For each={[{ value: "summary", label: t("screen.results.tabSummary") }, { value: "graphs", label: t("screen.results.tabGraphs") }, { value: "board", label: t("screen.results.tabBoard") }]}>
                {(tab) => (
                  <View class={`relative flex-row gap-1 justify-center items-center grow min-h-11 data-selected:text-accent-ink text-fg-muted`} role="tab" tabIndex={0} selected={props.tab === tab().value} data-selected={props.tab === tab().value} label={tab().label} onPress={(e) => props.onTab?.(e, tab().value)}>
                    <Show when={props.tab === tab().value}><View class={`absolute inset-x-2 bottom-0 h-0.75 rounded-capsule bg-accent pointer-events-none ${packClass("Tab", "indicator")}`} /></Show>
                    <Text>{tab().label}</Text>
                  </View>
                )}
              </For>
            </View>
          </Show>
          <Show when={props.tab === "summary"} fallback={
              <Show when={props.tab === "graphs"} fallback={
                  <Show when={props.tab === "board"}>
                    <View>{props.scores?.()}</View>
                  </Show>
              }>
                <View>{props.graph?.()}</View>
              </Show>
          }>
            <View class="grid grid-cols-[minmax(0,1.7fr)_minmax(0,1fr)] items-start gap-8 compact:grid-cols-1 compact:gap-4 short:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
              <View class={`grid grid-cols-fit-48.5 gap-3 compact:grid-cols-1 compact:gap-1 short:grid-cols-1 short:gap-1 ${packClass("List", "cascade")}`}>
                <For each={props.stats}>
                  {(s) => (
                  <View class={`rounded-card bg-plate-hi text-fg transition duration-control justify-between py-3 px-4 gap-2 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-dim compact:flex-row compact:min-h-0 compact:items-center compact:py-2 compact:px-3 short:flex-row short:min-h-0 short:items-center short:py-1 short:px-3 ${packClass("List", "item")}`} data-best={s().best}>
                    <View class="flex-row flex-wrap items-center justify-between gap-1 compact:justify-start compact:grow compact:shrink compact:basis-0 short:justify-start short:grow short:shrink short:basis-0">
                      <Text class={`tracking-label uppercase text-fg-muted text-xs font-semibold m-0 shrink`}>
                        {s().name}
                      </Text>
                      <Show when={s().best}>
                        <Text class={`text-xs font-bold tracking-label uppercase text-gold-fg bg-gold-hi rounded-control whitespace-nowrap py-0 px-1`}>
                          {t("screen.results.best")}
                        </Text>
                      </Show>
                    </View>
                    <Show when={s().count} fallback={
                        <Text class={`tabular-nums text-fg leading-flat font-bold whitespace-nowrap m-0 compact:text-4xl short:text-4xl ${packClass("Count", "roll")}`}>
                          {t("screen.results.StatValue", { format: s().format, value: s().format === 'percent' ? s().value * 100 : s().value })}
                        </Text>
                    }>
                      <Counter class={`text-fg leading-flat font-bold compact:text-4xl short:text-4xl`} value={s().value} from="0" format={s().format === 'percent' ? 'percent' : 'grouped'} decimals={s().decimals} />
                    </Show>
                  </View>
                  )}
                </For>
              </View>
              <Show when={props.rewards.length}>
                <View class="gap-2">
                  <Text class="text-xs font-semibold tracking-label uppercase text-fg-muted m-0">
                    {t("screen.results.rewards")}
                  </Text>
                  <View class={`flex-row flex-wrap gap-3 ${packClass("List", "cascade")}`}>
                    <For each={props.rewards}>
                      {(r) => (
                      <View class={`text-fg transition duration-control items-center w-29 rounded-button data-selected:inset-ring-regular data-selected:inset-ring-accent data-done:inset-ring-regular data-done:inset-ring-success active:inset-ring-regular active:inset-ring-accent disabled:text-fg-subtle disabled:scale-100 active:scale-press data-locked:scale-100 data-down:opacity-dim compact:w-24 shrink ${packClass("Reward", "pop")}`}>
                        <View class="relative rounded-card overflow-hidden bg-linear-to-b from-rim-strong via-rim via-45% to-rim-soft p-0.75 gap-2 size-26 compact:size-17">
                          <Art class="absolute bg-linear-to-b from-plate-hi to-plate-lo object-cover inset-0" kind="itemIcon" src={r().texture} />
                          <Show when={!r().texture && r().icon}>
                            <Icon class="absolute inset-0 m-auto size-14 compact:size-10 text-gold" name={r().icon ?? ""} />
                          </Show>
                        </View>
                        <Text class={`-mt-3 mb-0 text-xs font-bold tabular-nums text-fg bg-tile inset-shadow-lit whitespace-nowrap py-0 px-1 mx-0 ${packClass("Count", "roll")}`}>
                          {t("screen.results.Amount", { r_amount: formatCompact(r().amount) })}
                        </Text>
                        <Text class="mt-1 mb-0 w-full text-center text-caption font-semibold text-fg mx-0">
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
          <Show when={props.onReplay}>
            <Button class={`min-h-11 min-w-36 compact:min-w-0 compact:col-span-full compact:row-[2] shrink`} onPress={(e) => props.onReplay?.(e)}>
              <Text>{t("screen.results.replay")}</Text>
            </Button>
          </Show>
          <Show when={props.onAgain}>
            <Button class={`min-h-11 min-w-36 compact:min-w-0 shrink`} onPress={(e) => props.onAgain?.(e)}>
              <Text>{t("screen.results.again")}</Text>
            </Button>
          </Show>
          <Show when={props.onContinue}>
            <Button variant="primary" class={`min-h-11 min-w-36 compact:min-w-0 shrink`} onPress={(e) => props.onContinue?.(e)} onAction={{ confirm: (e) => props.onContinue?.(e) }}>
              <Text>{t("screen.results.continue")}</Text>
            </Button>
          </Show>
        </View>
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<ResultsScreenProps>> = {
  default: {"gained":1240,"grade":"A","level":18,"rewards":[{"amount":320,"id":"item-0","name":"Gold","texture":0,"icon":"coin"},{"amount":1,"id":"item-1","name":"Crate","texture":0,"icon":"bag"}],"stats":[{"best":true,"count":true,"id":"item-0","name":"Score","value":48210},{"best":false,"id":"item-1","name":"Kills","value":14},{"best":false,"format":"percent","id":"item-2","name":"Accuracy","value":0.62}],"tab":"summary","title":"Victory","won":true,"xp":0.45,"onReplay":press,"onAgain":press,"onContinue":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"results","layer":"menu","focus":"Panel/Footer/Continue","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
