import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import { HudTimer } from "../hud/hud-timer";
import "./racing.en.ogt";

export interface RacingLapHudProps {
  /** The lap the player is on, from 1; 0 before the start shows a dash. */
  lap: number;
  /** Laps in the race; 0 hides the lap's total (an open session). */
  laps: number;
  /** The player's place, from 1; 0 shows a dash. */
  position: number;
  /** Cars in the race; 0 hides the place. */
  racers: number;
  /** Places gained since the start as the game counts them: positive reads Up n places, negative Down n places (on data-trend), 0 hides it. */
  gained: number;
  /** The lap time running now, as the game formats it (1:24.618); empty shows the blank time. */
  current: string;
  /** The player's best lap, formatted, in the best lap's violet; empty shows the blank time. */
  best: string;
  /** The last lap, formatted; empty shows the blank time. */
  last: string;
  /** The split's name as the game words it (Sector 2); shown beside the delta. */
  sector: string;
  /** The split delta as the game words it (−0.412, +1.204), on a green tag ahead and a red one behind; empty hides the row. */
  split: string;
  /** ahead, behind or even: the delta's sense, on data-state and read with it. Default "even". */
  splitState?: string;
  /** The last lap was a new best: a Best lap tag stands on the last row. */
  freshBest: boolean;
  /** The player is on the final lap: the lap plate reads Final lap (data-final). */
  finalLap: boolean;
  /** Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it. Default -1. */
  timeLeft?: number;
  /** The checkpoint's whole time, for the timer's ring; 0 draws none. */
  timeTotal: number;
  /** Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out. Default 10. */
  warnAt?: number;
}

/**
 * The race's place and clock at the top left: the place large and slanted
 * over the field's size, the lap on a lit plate and the places gained
 * beside it; under them the lap time running, the last and best laps and
 * the split delta on a green or red tag. Every figure and time comes from
 * the game, formatted; read only.
 */
export function RacingLapHud(props: RacingLapHudProps) {
  return (
    <View class={`flex-col gap-y-[1.286rem] w-[17.857rem] max-w-full text-fg compact:gap-y-[0.571rem] compact:w-full`} label={t("racing.lap.name")} data-final={props.finalLap}>
      {/* Head */}
      <View class="flex-row items-end gap-x-[1.143rem] min-w-0 compact:gap-x-2">
        <Show when={props.racers > 0}>
          <View
            class="shrink-0 flex-row items-baseline"
            role="group"
            label={props.position > 0 ? t("racing.lap.positionValue", { position: props.position, racers: props.racers }) : t("racing.lap.position")}
          >
            <Text class={`text-[6rem] leading-[0.8] italic font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg compact:text-[3.143rem]`}>
              {props.position > 0 ? formatNumber(props.position) : t("racing.lap.pending")}
            </Text>
            <Text class={`text-[2.143rem] leading-[0.8] italic font-bold whitespace-nowrap text-fg-muted compact:text-[1.714rem]`}>{t("racing.lap.of", { total: props.racers })}</Text>
          </View>
        </Show>
        <View class="flex min-w-0 flex-col items-start gap-y-[0.571rem] pb-[0.286rem] compact:gap-y-[0.286rem] compact:pb-0">
          <Show when={props.laps > 0 || props.lap > 0}>
            <View
              class={`group/lap shrink-0 flex-row items-baseline gap-x-[0.571rem] py-[0.214rem] px-[0.643rem] bg-plate-hi nine-plate-cut ${packClass("Plate", "frame")}`}
              role="group"
              label={props.laps > 0 && props.lap > 0 ? t("racing.lap.lapValue", { lap: props.lap, laps: props.laps }) : t("racing.lap.lap")}
              data-final={props.finalLap}
            >
              <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg/78 group-data-final/lap:text-[#38e8ff]`}>
                {props.finalLap ? t("racing.lap.finalLap") : t("racing.lap.lap")}
              </Text>
              <Text class={`text-[1.071rem] font-bold tabular-nums whitespace-nowrap text-fg`}>{props.lap > 0 ? formatNumber(props.lap) : t("racing.lap.pending")}</Text>
              <Show when={props.laps > 0}>
                <Text class={`text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg-muted`}>{t("racing.lap.ofSpaced", { total: props.laps })}</Text>
              </Show>
            </View>
          </Show>
          <Show when={props.gained !== 0}>
            <View class="group/gained flex-row items-center gap-x-[0.357rem] max-w-full min-w-0" data-trend-down={props.gained < 0}>
              <View
                class={`shrink-0 size-[0.571rem] -rotate-45 border-l-2 border-b-2 border-[#38e8ff] group-data-trend-down/gained:rotate-45 group-data-trend-down/gained:border-t-2 group-data-trend-down/gained:border-r-2 group-data-trend-down/gained:border-l-0 group-data-trend-down/gained:border-b-0 group-data-trend-down/gained:border-[#ff4d5e]`}
              />
              <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-[#38e8ff] group-data-trend-down/gained:text-[#ff4d5e] compact:tracking-normal`}>
                {props.gained > 0 ? t("racing.lap.up", { n: props.gained }) : t("racing.lap.down", { n: -props.gained })}
              </Text>
            </View>
          </Show>
        </View>
      </View>
      {/* Times */}
      <View class="flex-col gap-y-1 compact:gap-y-[0.143rem]" role="list" label={t("racing.lap.times")}>
        <View class="flex-row items-baseline justify-between gap-x-2 min-w-0" role="listitem">
          <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg/78 compact:tracking-normal`}>{t("racing.lap.current")}</Text>
          <Text class={`text-[1.714rem] leading-[1.05] font-bold tabular-nums whitespace-nowrap text-fg`}>{props.current !== "" ? props.current : t("racing.lap.none")}</Text>
        </View>
        <View class="group/lastrow flex-row items-baseline justify-between gap-x-2 min-w-0 compact:hidden" role="listitem" data-fresh={props.freshBest}>
          <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:tracking-normal`}>{t("racing.lap.last")}</Text>
          <View class="flex-row items-baseline gap-x-[0.429rem]">
            <Show when={props.freshBest}>
              <Text class={`shrink-0 py-0 px-[0.357rem] text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-epic bg-[#060910e0] nine-chip-cut ${packClass("Chip", "frame")}`}>{t("racing.lap.newBest")}</Text>
            </Show>
            <Text class={`text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg-muted group-data-fresh/lastrow:text-epic`}>{props.last !== "" ? props.last : t("racing.lap.none")}</Text>
          </View>
        </View>
        <View class="flex-row items-baseline justify-between gap-x-2 min-w-0" role="listitem">
          <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-epic compact:tracking-normal`}>{t("racing.lap.best")}</Text>
          <Text class={`text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-epic`}>{props.best !== "" ? props.best : t("racing.lap.none")}</Text>
        </View>
        <Show when={props.split !== ""}>
          <View
            class="flex-row items-baseline justify-between gap-x-2 min-w-0"
            role="listitem"
            label={
              (props.splitState ?? "even") === "ahead"
                ? t("racing.lap.splitAhead", { sector: props.sector, delta: props.split })
                : (props.splitState ?? "even") === "behind"
                  ? t("racing.lap.splitBehind", { sector: props.sector, delta: props.split })
                  : t("racing.lap.splitEven", { sector: props.sector, delta: props.split })
            }
            data-ahead={(props.splitState ?? "even") === "ahead"}
            data-behind={(props.splitState ?? "even") === "behind"}
          >
            <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg/78 compact:tracking-normal`}>{props.sector !== "" ? props.sector : t("racing.lap.split")}</Text>
            <Text
              class={`shrink-0 py-0 px-[0.429rem] text-[1.071rem] font-bold tabular-nums whitespace-nowrap text-fg bg-[#060910e0] nine-chip-cut data-ahead:bg-[#55e39a] data-ahead:text-[#0d2417] data-behind:bg-[#ff4d5e] data-behind:text-fg ${packClass("Chip", "frame")}`}
              data-ahead={(props.splitState ?? "even") === "ahead"}
              data-behind={(props.splitState ?? "even") === "behind"}
            >
              {props.split}
            </Text>
          </View>
        </Show>
      </View>
      <Show when={(props.timeLeft ?? -1) >= 0}>
        <View class="self-start">
          <HudTimer seconds={props.timeLeft ?? 0} total={props.timeTotal} countdown warnAt={props.warnAt ?? 10} paused={false} label={t("racing.lap.timeLeft")} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingLapHudProps>> = {
  default: { "lap": 2, "laps": 3, "position": 3, "racers": 8, "gained": 2, "current": "1:24.618", "best": "1:57.790", "last": "1:58.204", "sector": "Sector 2", "split": "−0.412", "splitState": "ahead", "freshBest": false, "finalLap": false, "timeLeft": -1, "timeTotal": 0 },
  leading: { "lap": 3, "laps": 3, "position": 1, "racers": 8, "gained": 5, "current": "0:12.660", "best": "1:20.871", "last": "1:20.871", "sector": "Sector 1", "split": "+0.204", "splitState": "behind", "freshBest": true, "finalLap": true, "timeLeft": -1, "timeTotal": 0 },
  last: { "lap": 2, "laps": 5, "position": 8, "racers": 8, "gained": -3, "current": "0:48.207", "best": "1:31.904", "last": "1:33.116", "sector": "Sector 3", "split": "+2.950", "splitState": "behind", "freshBest": false, "finalLap": false, "timeLeft": -1, "timeTotal": 0 },
  grid: { "lap": 0, "laps": 3, "position": 0, "racers": 8, "gained": 0, "current": "", "best": "", "last": "", "sector": "", "split": "", "splitState": "even", "freshBest": false, "finalLap": false, "timeLeft": -1, "timeTotal": 0 },
  checkpoint: { "lap": 4, "laps": 0, "position": 0, "racers": 0, "gained": 0, "current": "3:41.090", "best": "0:55.020", "last": "0:57.311", "sector": "Checkpoint 6", "split": "0.000", "splitState": "even", "freshBest": false, "finalLap": false, "timeLeft": 8.4, "timeTotal": 30 },
  overflow: { "lap": 118, "laps": 150, "position": 24, "racers": 24, "gained": -23, "current": "1:02:13.456", "best": "1:58:59.999", "last": "10:21:04.120", "sector": "Sector 14 · Nordschleife Hatzenbach", "split": "+12:31.877", "splitState": "behind", "freshBest": true, "finalLap": true, "timeLeft": -1, "timeTotal": 0 },
};
