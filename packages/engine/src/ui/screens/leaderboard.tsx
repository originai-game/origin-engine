import type { Texture } from "@origingame/engine";
import { Scroll, For, Show, Text, type UiEvent, View, formatDuration, t } from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./leaderboard.en.ogt";

/** English ordinal (1st, 2nd, 23rd): formatOrdinal has no TSX surface. */
function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] || s[v] || s[0]}`;
}

export interface LeaderboardScreenProps {
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
}

export function LeaderboardScreen(props: LeaderboardScreenProps) {
  return (
    <View class={`bg-linear-to-r from-[#04060ce5] via-[#04060c9e] via-55% to-[#04060ccc] text-fg grow shrink basis-0 p-8 gap-3 compact:p-3 compact:gap-2 disabled:text-fg-subtle`}>
      <View class={`w-full max-w-264 flex-wrap items-center gap-y-2 gap-x-3 p-0 mx-auto`}>
        <Button class="shrink" onPress={(e) => props.onBack?.(e)}><Text>{t("screen.common.back")}</Text></Button>
        <Text class={`font-display text-[3.286rem] font-bold leading-flat tracking-sm uppercase text-fg grow shrink m-0 compact:text-[2rem]`}>
          {t("screen.leaderboard.title")}
        </Text>
        <Button class="h-7 px-3 compact:h-8 shrink" onPress={(e) => props.onPeriod?.(e)}>
          <Text>{props.period}</Text>
        </Button>
      </View>
      <View class="flex-row flex-wrap items-center w-fit max-w-264 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none p-0.75 mx-auto gap-0 compact:w-full" focus-group="wrap">
        <Button class="uppercase tracking-wide data-selection-off:text-fg/66 data-selection-on:text-fg data-selection-on:bg-linear-to-t data-selection-on:from-[#38e8ff33] data-selection-on:to-transparent data-selection-on:bg-transparent data-selection-on:shadow-[inset_0px_-2.288px_0px_#38e8ff] compact:tracking-sm compact:grow compact:shrink compact:basis-0 compact:min-w-0 compact:px-1" data-selection-off={props.tab !== "global"} data-selection-on={props.tab === "global"}>
          <Text class="compact:text-ellipsis">{t("screen.leaderboard.global")}</Text>
        </Button>
        <Button class="uppercase tracking-wide data-selection-off:text-fg/66 data-selection-on:text-fg data-selection-on:bg-linear-to-t data-selection-on:from-[#38e8ff33] data-selection-on:to-transparent data-selection-on:bg-transparent data-selection-on:shadow-[inset_0px_-2.288px_0px_#38e8ff] compact:tracking-sm compact:grow compact:shrink compact:basis-0 compact:min-w-0 compact:px-1" data-selection-off={props.tab !== "friends"} data-selection-on={props.tab === "friends"}>
          <Text class="compact:text-ellipsis">{t("screen.leaderboard.friends")}</Text>
        </Button>
        <Button class="uppercase tracking-wide data-selection-off:text-fg/66 data-selection-on:text-fg data-selection-on:bg-linear-to-t data-selection-on:from-[#38e8ff33] data-selection-on:to-transparent data-selection-on:bg-transparent data-selection-on:shadow-[inset_0px_-2.288px_0px_#38e8ff] compact:tracking-sm compact:grow compact:shrink compact:basis-0 compact:min-w-0 compact:px-1" data-selection-off={props.tab !== "league"} data-selection-on={props.tab === "league"}>
          <Text class="compact:text-ellipsis">{t("screen.leaderboard.league")}</Text>
        </Button>
      </View>
      <Show when={props.tab === "league"}>
        <View class={`w-full max-w-264 items-center rounded-tl-[6.864px] rounded-tr-none rounded-br-[6.864px] rounded-bl-none flex-wrap bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] py-2 px-3 mx-auto gap-3`}>
          <Text class="text-[1.714rem] font-bold tracking-sm uppercase text-gold m-0 shrink">
            {props.league.name}
          </Text>
          <Text class="grow shrink text-caption text-fg/66 m-0">
            {t("screen.leaderboard.Lines", { league_demote: props.league.demote, league_promote: props.league.promote })}
          </Text>
          <Text class="text-[0.857rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] whitespace-nowrap py-0 px-1 m-0 shrink">
            {t("screen.leaderboard.Ends", { league_ends: formatDuration(props.league.ends) })}
          </Text>
        </View>
      </Show>
      <Scroll class="w-full max-w-264 grow shrink basis-0 mx-auto">
        <View class="gap-0.5" focus-group="stop">
          <View>{props.board?.()}</View>
          <For each={props.rows}>
            {(r) => (
            <View class={`group/row rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-linear-to-r from-[#171f30c7] to-[#0b101c8c] text-fg transition duration-317 grid grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-x-3 pr-3 pl-1 py-1 hover:bg-transparent hover:bg-linear-to-r hover:from-[#2a3750e5] hover:to-[#171f30b2] hover:shadow-[inset_0px_0px_0px_1px_#ffffff4d] data-mine-you:bg-transparent data-mine-you:bg-linear-to-r data-mine-you:from-[#38e8ff3d] data-mine-you:via-[#38e8ff14] data-mine-you:via-55% data-mine-you:to-[#38e8ff08] data-mine-you:text-fg data-mine-you:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] data-promote:shadow-[inset_3px_0px_0px_#55e39a] data-down:opacity-45 narrow:grid-cols-[auto_auto_minmax(0,1fr)] narrow:gap-y-0 compact:gap-x-2 active:scale-99`} onPress={(e) => props.onRow?.(e, r().id)} tabIndex={0} data-mine-other={!r().you} data-mine-you={r().you} data-promote={props.tab === "league" && r().rank <= props.league.promote} data-place-1={(r().rank <= 3 ? r().rank : 0) === 1} data-place-2={(r().rank <= 3 ? r().rank : 0) === 2} data-place-3={(r().rank <= 3 ? r().rank : 0) === 3}>
              <Text class={`text-center min-w-[3.714rem] leading-[2.714rem] whitespace-nowrap text-[1.143rem] font-bold tabular-nums bg-white/14 rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none py-0 px-1 m-0 compact:min-w-[2.714rem] compact:leading-[2.286rem] group-data-place-1/row:bg-none group-data-place-1/row:bg-gold group-data-place-1/row:text-[#1e1604] group-data-place-2/row:bg-none group-data-place-2/row:bg-[#d0daec] group-data-place-2/row:text-[#1e1604] group-data-place-3/row:bg-none group-data-place-3/row:bg-[#e69662] group-data-place-3/row:text-[#1e1604]`}>
                {t("screen.leaderboard.RowRank", { value: ordinal(r().rank) })}
              </Text>
              <Art class="size-8 compact:size-6" kind="emblem" src={r().texture} />
              <View class="flex-row items-center gap-1">
                <Text class="text-[1.143rem] font-semibold whitespace-nowrap m-0 overflow-hidden shrink">
                  {r().name}
                </Text>
                <Show when={r().you}>
                  <Text class="text-[0.857rem] font-bold tracking-[0.14em] uppercase text-[#38e8ff] bg-[#060910e0] shadow-[inset_0px_0px_0px_1px_#38e8ff73,inset_0px_2px_0px_#38e8ff] whitespace-nowrap py-0 px-1">
                    {t("screen.leaderboard.you")}
                  </Text>
                </Show>
              </View>
              <Text class={`tabular-nums text-fg text-[1.714rem] font-bold whitespace-nowrap text-end m-0 group-data-place-1/row:text-gold group-data-mine-you/row:text-fg compact:text-[1.143rem] narrow:text-start`}>
                {t("screen.leaderboard.RowScore", { value: r().score })}
              </Text>
            </View>
            )}
          </For>
        </View>
      </Scroll>
      <View class={`w-full max-w-264 items-center pr-3 pl-1 rounded-tl-[6.864px] rounded-tr-none rounded-br-[6.864px] rounded-bl-none shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] text-fg py-2 mx-auto gap-3`}>
        <Text class={`text-center min-w-[3.714rem] leading-[2.714rem] whitespace-nowrap text-[1.143rem] font-bold tabular-nums bg-white/14 rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none py-0 px-1 m-0 compact:min-w-[2.714rem] compact:leading-[2.286rem] shrink`}>
          {t("screen.leaderboard.MeRank", { value: ordinal(props.me.rank) })}
        </Text>
        <Text class="text-[1.143rem] font-semibold whitespace-nowrap shrink m-0 overflow-hidden">
          {props.me.name}
        </Text>
        <Text class="text-[0.857rem] font-bold tracking-[0.14em] uppercase text-[#38e8ff] bg-[#060910e0] shadow-[inset_0px_0px_0px_1px_#38e8ff73,inset_0px_2px_0px_#38e8ff] whitespace-nowrap mr-auto py-0 px-1">
          {t("screen.leaderboard.you")}
        </Text>
        <Text class={`tabular-nums text-fg text-[2.429rem] font-bold whitespace-nowrap text-end m-0 compact:text-[1.714rem] shrink`}>
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
