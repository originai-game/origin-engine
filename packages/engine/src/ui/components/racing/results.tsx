import { For, Scroll, Show, Text, View, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingResultsProps {
  /** The page's heading (the event's name); empty takes the kit's (Results). */
  title?: string;
  /** The finishing order as the game classifies it, keyed rows of { id, position, name, time, points, fastest, player, state }: time the race time or gap as the game words it; points as text (empty hides it); fastest marks the fastest lap; player the local player's row; state finished, dnf or dsq (dnf and dsq are named in words and show no place or time). */
  rows?: ReadonlyArray<{ id: string; position: number; name: string; time: string; points: string; fastest: boolean; player: boolean; state: string }>;
  /** The race's fastest lap time, formatted; empty hides the line under the table. */
  bestLap: string;
  /** Who set the fastest lap; empty leaves the name out of the line. */
  bestBy: string;
  /** Retry is offered (false: disabled). */
  canRetry: boolean;
  /** A row is the player's: the hero beside the tower shows its place, time and points; false hides it. */
  hasPlayer: boolean;
  /** Retry (also the details action, X or pad Y). */
  onRetry?: (e: UiEvent) => void;
  /** Continue. */
  onContinue?: (e: UiEvent) => void;
}

/** English ordinal (1st, 2nd, 23rd): formatOrdinal has no TSX surface. */
function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] || s[0] || s[v]}`;
}

const hudLabel = `text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7]`;
const actionButton = `min-h-11 min-w-[8.571rem] flex-row items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d] hover:bg-[#2a3242eb] disabled:opacity-45 compact:flex-1 compact:min-w-0 ${packClass("Button", "plate")}`;
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] leading-[1.5] py-0 px-1`;

/**
 * The classification after the flag: place, driver, time or gap, points
 * and the fastest-lap mark per row, the player's row marked You, cars
 * that did not finish or were disqualified named so. The game computes
 * every time and point. Continue and Retry (also the details action, X
 * or pad Y) report once; a Retry the game refuses is disabled.
 */
export function RacingResults(props: RacingResultsProps) {
  const heading = () => ((props.title ?? "") !== "" ? (props.title ?? "") : t("racing.results.title"));
  const rows = () => props.rows ?? [];
  const playerRow = () => rows().find((r) => r.player);
  return (
    <View class={`flex-col w-full h-full text-fg`} label={heading()}>
      <View class="flex-row w-full shrink-0">
        <Text class={`flex-1 min-w-0 text-[3.286rem] leading-[1.05] font-bold tracking-[0.04em] uppercase text-fg compact:text-[2rem]`}>{heading()}</Text>
      </View>
      <Show when={rows().length} fallback={
        <Text class={`flex-1 min-h-0 text-center`} role="status">{t("racing.results.empty")}</Text>
      }>
        <View class={`grid grid-cols-[minmax(0,0.8fr)_minmax(0,1.4fr)] gap-x-6 gap-y-3 items-start min-h-0 w-full flex-1 compact:grid-cols-1`}>
          <View class={`flex-col gap-2 min-w-0 data-empty:hidden compact:flex-row compact:flex-wrap compact:items-center`} role="group" label={t("racing.results.you")} data-empty={!props.hasPlayer}>
            <Show when={playerRow()} keyed>
              {(r) => (
                <>
                  <Text class={`text-[6rem] leading-[0.9] font-bold italic whitespace-nowrap text-[#38e8ff] text-stroke-3 text-stroke-bg compact:text-[3.143rem]`}>
                    {r().state === "dnf" || r().state === "dsq" ? (r().state === "dsq" ? t("racing.results.dsq") : t("racing.results.dnf")) : ordinal(r().position)}
                  </Text>
                  <View class={`flex-row flex-wrap items-baseline justify-between gap-x-3 py-2 px-3 bg-[#1e2637e5] nine-plate-cut ${packClass("Plate", "frame")}`}>
                    <Text class={hudLabel}>{t("racing.results.time")}</Text>
                    <Text class={`text-[2.429rem] font-bold tabular-nums whitespace-nowrap text-fg compact:text-[1.714rem]`}>{r().time}</Text>
                  </View>
                  <Show when={r().points !== ""}>
                    <View class={`flex-row flex-wrap items-baseline justify-between gap-x-3 py-2 px-3 bg-[#1e2637e5] nine-plate-cut ${packClass("Plate", "frame")}`}>
                      <Text class={hudLabel}>{t("racing.results.points")}</Text>
                      <Text class={`text-[2.429rem] font-bold tabular-nums whitespace-nowrap text-gold compact:text-[1.714rem]`}>{r().points}</Text>
                    </View>
                  </Show>
                </>
              )}
            </Show>
          </View>
          <View class="flex-col flex-1 min-w-0 min-h-0">
            <View class="flex-row items-center gap-4 py-[0.143rem] pr-3 shrink-0">
              <Text class={`shrink-0 w-[2.286rem] text-center text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7]`}>{t("racing.results.position")}</Text>
              <Text class={`flex-1 text-left text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7]`}>{t("racing.results.driver")}</Text>
              <Text class={`shrink-0 w-[7.886rem] text-right text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] compact:w-[6.143rem] compact:tracking-[0.04em] compact:text-[0.857rem]`}>{t("racing.results.time")}</Text>
              <Text class={`shrink-0 w-[3.429rem] text-right text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] compact:w-[2.571rem] compact:tracking-[0.04em] compact:text-[0.857rem]`}>{t("racing.results.points")}</Text>
            </View>
            <Scroll class="flex-1 min-h-0 w-full">
              <View class="flex-col gap-[0.143rem] w-full" role="list" label={heading()}>
                <For each={rows()} key={(r) => r.id}>
                  {(r, i) => (
                    <View
                      class={`flex-row items-center gap-4 min-h-11 py-[0.143rem] pr-3 bg-[#0d1526db] nine-plate-cut text-fg data-dnf:opacity-60 data-dsq:opacity-60 data-player:bg-[#38e8ff] data-player:text-[#141a02] compact:gap-[0.571rem] compact:py-[0.143rem] compact:pr-[0.286rem]`}
                      role="listitem"
                      label={r().name}
                      current={r().player}
                      data-first={i() === 0}
                      data-fastest={r().fastest}
                      data-player={r().player}
                      data-leader={r().position === 1 && r().state === "finished"}
                      data-dnf={r().state === "dnf"}
                      data-dsq={r().state === "dsq"}
                    >
                      <Text class={`shrink-0 self-stretch w-[2.286rem] text-center leading-[3.143rem] text-[1.143rem] font-bold tabular-nums tracking-normal text-fg bg-[#ffffff24] data-leader:bg-gold data-leader:text-[#231c02] data-player:bg-[#00000033] data-player:text-[#141a02] compact:text-[0.929rem]`}>
                        {r().state === "dnf" || r().state === "dsq" ? t("racing.results.unplaced") : formatNumber(r().position)}
                      </Text>
                      <View class="flex-1 min-w-0 flex-row flex-wrap items-center gap-x-0.5 gap-y-0.5">
                        <Text class={`flex-1 basis-full text-left text-[1.071rem] font-semibold text-fg compact:text-[0.929rem]`}>{r().name}</Text>
                        <Show when={r().player}>
                          <Text class={`shrink-0 py-0 px-1 text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap bg-[#060910e0] nine-chip-cut text-fg compact:tracking-[0.04em] compact:text-[0.857rem] ${packClass("Chip", "frame")}`}>{t("racing.results.you")}</Text>
                        </Show>
                        <Show when={r().fastest}>
                          <Text class={`shrink-0 py-0 px-1 text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap bg-[#060910e0] nine-chip-cut text-epic ${packClass("Chip", "frame")}`} label={t("racing.results.fastest")}>{t("racing.results.fastest")}</Text>
                        </Show>
                      </View>
                      <Show
                        when={r().state === "dnf" || r().state === "dsq"}
                        fallback={
                          <Text class={`shrink-0 w-[7.886rem] text-right text-[1.714rem] font-bold tabular-nums whitespace-nowrap text-fg compact:w-[6.143rem] compact:text-[0.929rem]`}>{r().time}</Text>
                        }
                      >
                        <Text class={`shrink-0 ml-auto py-0 px-1 text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap bg-[#060910e0] nine-chip-cut text-[#ff4d5e] ${packClass("Chip", "frame")}`} label={r().state === "dsq" ? t("racing.results.dsqName") : t("racing.results.dnfName")}>
                          {r().state === "dsq" ? t("racing.results.dsq") : t("racing.results.dnf")}
                        </Text>
                      </Show>
                      <Text class={`shrink-0 w-[3.429rem] text-right text-[1.714rem] font-bold tabular-nums whitespace-nowrap text-gold compact:w-[2.571rem] compact:text-[0.929rem]`}>{r().points}</Text>
                    </View>
                  )}
                </For>
              </View>
            </Scroll>
            <Show when={props.bestLap !== ""}>
              <Text class={`shrink-0 pt-2 tracking-normal text-fg`}>
                {props.bestBy !== "" ? t("racing.results.bestLap", { time: props.bestLap, name: props.bestBy }) : t("racing.results.bestLapTime", { time: props.bestLap })}
              </Text>
            </Show>
          </View>
        </View>
      </Show>
      <View class="flex-row w-full shrink-0 justify-end compact:justify-stretch">
        <View class={actionButton} role="button" disabled={!props.canRetry} label={t("racing.results.retry")} onPress={(e) => props.onRetry?.(e)} onAction={{ details: (e) => props.onRetry?.(e) }}>
          <Text glyph="details" class={keyCap} />
          <Text>{t("racing.results.retry")}</Text>
        </View>
        <View class={`min-h-11 min-w-[8.571rem] flex-row items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d] hover:bg-[#2a3242eb] disabled:opacity-45 compact:flex-1 compact:min-w-0 bg-[#38e8ff] text-[#141a02] shadow-none ${packClass("Button", "plate", "primary")}`} role="button" label={t("racing.results.continue")} onPress={(e) => props.onContinue?.(e)}>
          <Text>{t("racing.results.continue")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingResultsProps>> = {
  default: { "title": "", "rows": [{ "id": "marlow", "position": 1, "name": "Marlow", "time": "24:31.207", "points": "25", "fastest": false, "player": false, "state": "finished" }, { "id": "you", "position": 2, "name": "Rhea Vance", "time": "+0.207", "points": "18", "fastest": true, "player": true, "state": "finished" }, { "id": "okafor", "position": 3, "name": "Okafor", "time": "+2.940", "points": "15", "fastest": false, "player": false, "state": "finished" }, { "id": "sato", "position": 4, "name": "Sato", "time": "+1 lap", "points": "12", "fastest": false, "player": false, "state": "finished" }, { "id": "brandt", "position": 5, "name": "Brandt", "time": "", "points": "", "fastest": false, "player": false, "state": "dnf" }], "bestLap": "1:20.871", "bestBy": "Rhea Vance", "canRetry": true, "hasPlayer": true },
  empty: { "title": "", "rows": [], "bestLap": "", "bestBy": "", "canRetry": true, "hasPlayer": false },
  disabled: { "title": "Coastal Cup · Final", "rows": [{ "id": "you", "position": 1, "name": "Rhea Vance", "time": "8:02.114", "points": "", "fastest": true, "player": true, "state": "finished" }, { "id": "okafor", "position": 2, "name": "Okafor", "time": "", "points": "", "fastest": false, "player": false, "state": "dsq" }], "bestLap": "", "bestBy": "", "canRetry": false, "hasPlayer": true },
  overflow: { "title": "Grand Touring Championship · Round 11 · Nordschleife Endurance", "rows": [{ "id": "r1", "position": 1, "name": "Maximilian Vandenberghe-Okonkwo", "time": "2:01:31.207", "points": "25", "fastest": false, "player": false, "state": "finished" }, { "id": "r2", "position": 2, "name": "Okafor", "time": "+0.884", "points": "18", "fastest": false, "player": false, "state": "finished" }, { "id": "r3", "position": 3, "name": "Lindqvist", "time": "+1.020", "points": "15", "fastest": true, "player": false, "state": "finished" }, { "id": "r4", "position": 4, "name": "Sato", "time": "+2.310", "points": "12", "fastest": false, "player": false, "state": "finished" }, { "id": "r5", "position": 5, "name": "Brandt", "time": "+3.007", "points": "10", "fastest": false, "player": false, "state": "finished" }, { "id": "r6", "position": 6, "name": "Duarte", "time": "+4.551", "points": "8", "fastest": false, "player": false, "state": "finished" }, { "id": "r7", "position": 7, "name": "Nakamura", "time": "+5.902", "points": "6", "fastest": false, "player": false, "state": "finished" }, { "id": "r8", "position": 8, "name": "Haddad", "time": "+7.118", "points": "4", "fastest": false, "player": false, "state": "finished" }, { "id": "r9", "position": 9, "name": "Rhea Vance", "time": "+8.640", "points": "2", "fastest": false, "player": true, "state": "finished" }, { "id": "r10", "position": 10, "name": "Moreau", "time": "+9.215", "points": "1", "fastest": false, "player": false, "state": "finished" }, { "id": "r11", "position": 11, "name": "Ferreira", "time": "+1 lap", "points": "", "fastest": false, "player": false, "state": "finished" }, { "id": "r12", "position": 12, "name": "Abernathy-Whitcombe", "time": "+2 laps", "points": "", "fastest": false, "player": false, "state": "finished" }, { "id": "r13", "position": 13, "name": "Petrov", "time": "", "points": "", "fastest": false, "player": false, "state": "dnf" }, { "id": "r14", "position": 14, "name": "Laurent", "time": "", "points": "", "fastest": false, "player": false, "state": "dsq" }], "bestLap": "8:14.390", "bestBy": "Lindqvist", "canRetry": true, "hasPlayer": true },
};
