import type { Texture } from "@origingame/engine";
import {Scroll, For, Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import { Tab, Tabs } from "../controls/tabs";
import "./leaderboard.en.ogt";

/** English ordinal (1st, 2nd, 23rd): formatOrdinal has no TSX surface. */
function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] || s[v] || s[0]}`;
}

export interface LeaderboardScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** global, friends or league. */
  tab: string;
  /** The period, as text (This week). */
  period: string;
  /** The top rows: { rank, name, score, you (bool), texture }. */
  rows: ReadonlyArray<{ id: string; name: string; rank: number; score: number; texture: Texture | 0; you: boolean }>;
  /** The stable collection key selected now. */
  selected: string;
  /** The player's row: { rank, name, score }. */
  me: Record<string, any>;
  /** The league: { name, promote (rank), demote (rank), ends (seconds) }. */
  league: Record<string, any>;
  /** Content the caller supplies (was the board snippet). */
  board?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The period event, with the row's key where it comes from a row. */
  onPeriod?: (e: UiEvent, key?: string) => void;
  /** The row event, with the row's key where it comes from a row. */
  onRow?: (e: UiEvent, key?: string) => void;
  /** A tab pressed, with its key (global, friends or league): the game sets `tab`. */
  onTab?: (e: UiEvent, key?: string) => void;
}

/**
 * A leaderboard: global, friends and league tabs, the period, the top rows
 * with the player's own marked, the player's row pinned, and the league's
 * promotion and demotion lines. The game owns the rows.
 */
export function LeaderboardScreen(props: LeaderboardScreenProps) {
  return (
    <View class={`absolute inset-0 text-fg grow shrink basis-0 p-8 gap-3 compact:p-3 compact:gap-2 disabled:text-fg-subtle ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <View class={`absolute bg-linear-to-r from-shade-deep via-shade-deep via-55% to-shade-deep inset-0`} />
      <View class={`w-full max-w-264 flex-row flex-wrap items-center gap-y-2 gap-x-3 p-0 mx-auto`}>
        <Button class={`shrink`} onPress={(e) => props.onBack?.(e)}><Text>{t("screen.common.back")}</Text></Button>
        <Text class={`font-display text-display-xl font-bold leading-flat tracking-sm uppercase text-fg grow shrink m-0 compact:text-display-sm`}>
          {t("screen.leaderboard.title")}
        </Text>
        <Button class={`h-7 px-3 compact:h-8 shrink`} onPress={(e) => props.onPeriod?.(e)}>
          <Text>{props.period}</Text>
        </Button>
      </View>
      <Tabs class={`w-fit max-w-264 mx-auto px-1 rounded-field compact:w-full ${packClass("Plate", "frame")}`} label={t("screen.leaderboard.title")}>
        <Tab class="uppercase tracking-wide compact:tracking-sm compact:grow compact:basis-0 compact:min-w-0 compact:px-1" selected={props.tab === "global"} onPress={(e) => props.onTab?.(e, "global")}>
          <Text class="truncate">{t("screen.leaderboard.global")}</Text>
        </Tab>
        <Tab class="uppercase tracking-wide compact:tracking-sm compact:grow compact:basis-0 compact:min-w-0 compact:px-1" selected={props.tab === "friends"} onPress={(e) => props.onTab?.(e, "friends")}>
          <Text class="truncate">{t("screen.leaderboard.friends")}</Text>
        </Tab>
        <Tab class="uppercase tracking-wide compact:tracking-sm compact:grow compact:basis-0 compact:min-w-0 compact:px-1" selected={props.tab === "league"} onPress={(e) => props.onTab?.(e, "league")}>
          <Text class="truncate">{t("screen.leaderboard.league")}</Text>
        </Tab>
      </Tabs>
      <Show when={props.tab === "league"}>
        <View class={`w-full max-w-264 flex-row items-center nine-panel-cut flex-wrap bg-plate-hi py-2 px-3 mx-auto gap-3 ${packClass("Panel", "frame")}`}>
          <Text class="text-title font-bold tracking-sm uppercase text-gold m-0 shrink">
            {props.league.name}
          </Text>
          <Text class="grow shrink text-caption text-fg-muted m-0">
            {t("screen.leaderboard.Lines", { league_demote: props.league.demote, league_promote: props.league.promote })}
          </Text>
          <Text class="text-xs font-bold tracking-label uppercase text-fg bg-tile inset-shadow-lit whitespace-nowrap py-0 px-1 m-0 shrink">
            {t("screen.leaderboard.Ends", { league_ends: formatDuration(props.league.ends) })}
          </Text>
        </View>
      </Show>
      <Scroll class="w-full max-w-264 shrink min-h-0 mx-auto">
        <View class={`gap-0.5 ${packClass("List", "cascade")}`} focus-group="stop">
          <View>{props.board?.()}</View>
          <For each={props.rows}>
            {(r) => (
            <View class={`group/row nine-plate-cut bg-layer-2 text-fg transition duration-317 grid grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-x-3 pr-3 pl-1 py-1 hover:bg-surface-hover data-mine-you:text-fg data-down:opacity-45 compact:gap-x-2 active:scale-99 ${r().you ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} ${packClass("List", "item")}`} onPress={(e) => props.onRow?.(e, r().id)} tabIndex={0} data-mine-other={!r().you} data-mine-you={r().you} data-promote={props.tab === "league" && r().rank <= props.league.promote} data-place-1={(r().rank <= 3 ? r().rank : 0) === 1} data-place-2={(r().rank <= 3 ? r().rank : 0) === 2} data-place-3={(r().rank <= 3 ? r().rank : 0) === 3}>
              <Text class={`text-center min-w-13 leading-9.5 whitespace-nowrap text-base font-bold tabular-nums nine-chip-cut bg-layer-2 py-0 px-1 m-0 compact:min-w-9.5 compact:leading-8 group-data-place-1/row:bg-none group-data-place-1/row:bg-gold group-data-place-1/row:nine-chip-cut group-data-place-1/row:text-gold-fg group-data-place-2/row:bg-none group-data-place-2/row:bg-fg group-data-place-2/row:nine-chip-cut group-data-place-2/row:text-gold-fg group-data-place-3/row:bg-none group-data-place-3/row:bg-legendary group-data-place-3/row:nine-chip-cut group-data-place-3/row:text-legendary-fg ${r().rank <= 3 ? "" : packClass("Chip", "frame")}`}>
                {t("screen.leaderboard.RowRank", { value: ordinal(r().rank) })}
              </Text>
              <Art class="size-8 compact:size-6" kind="emblem" src={r().texture} />
              <View class="flex-row items-center gap-1">
                <Text class="text-base font-semibold whitespace-nowrap m-0 overflow-hidden shrink">
                  {r().name}
                </Text>
                <Show when={r().you}>
                  <Text class="text-xs font-bold tracking-label uppercase text-hud-accent-ink bg-tile inset-shadow-overline inset-shadow-hud-accent inset-ring-1 inset-ring-hud-accent whitespace-nowrap py-0 px-1">
                    {t("screen.leaderboard.you")}
                  </Text>
                </Show>
              </View>
              <Text class={`tabular-nums text-fg text-title font-bold whitespace-nowrap text-end m-0 group-data-place-1/row:text-gold group-data-mine-you/row:text-fg compact:text-base`}>
                {t("screen.leaderboard.RowScore", { value: r().score })}
              </Text>
            </View>
            )}
          </For>
        </View>
      </Scroll>
      <View class={`w-full max-w-264 flex-row items-center pr-3 pl-1 nine-plate-cut bg-glass text-fg py-2 mx-auto gap-3 ${packClass("Plate", "frame")}`}>
        <Text class={`text-center min-w-13 leading-9.5 whitespace-nowrap text-base font-bold tabular-nums nine-plate-cut bg-layer-3 py-0 px-1 m-0 compact:min-w-9.5 compact:leading-8 shrink ${packClass("Chip", "frame")}`}>
          {t("screen.leaderboard.MeRank", { value: ordinal(props.me.rank) })}
        </Text>
        <Text class="text-base font-semibold whitespace-nowrap shrink m-0 overflow-hidden">
          {props.me.name}
        </Text>
        <Text class="text-xs font-bold tracking-label uppercase text-hud-accent-ink bg-tile inset-shadow-overline inset-shadow-hud-accent inset-ring-1 inset-ring-hud-accent whitespace-nowrap mr-auto py-0 px-1">
          {t("screen.leaderboard.you")}
        </Text>
        <Text class={`tabular-nums text-fg text-display-md font-bold whitespace-nowrap text-end m-0 compact:text-title shrink`}>
          {t("screen.leaderboard.MeScore", { value: props.me.score })}
        </Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<LeaderboardScreenProps>> = {
  default: {"league":{"demote":25,"ends":259200,"name":"Gold league","promote":10},"me":{"name":"You","rank":1284,"score":402110},"period":"This week","rows":[{"id":"item-0","name":"Nyx","rank":1,"score":982140,"texture":0,"you":false},{"id":"item-1","name":"Ferro","rank":2,"score":951300,"texture":0,"you":false},{"id":"item-2","name":"Lumen","rank":3,"score":948020,"texture":0,"you":false},{"id":"item-3","name":"Quill","rank":4,"score":902210,"texture":0,"you":false}],"selected":"","tab":"global"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Profile","back":"Back","tabNext":"Next tab","tabPrev":"Previous tab"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"leaderboard","layer":"menu","focus":"Body/List","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
