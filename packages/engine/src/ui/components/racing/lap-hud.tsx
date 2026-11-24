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
    <View class={`flex-col gap-y-4.5 w-62.5 max-w-full text-fg compact:gap-y-2 compact:w-full`} label={t("racing.lap.name")} data-final={props.finalLap}>
      {/* Head */}
      <View class="flex-row items-end gap-x-4 min-w-0 compact:gap-x-2">
        <Show when={props.racers > 0}>
          <View
            class="shrink-0 flex-row items-baseline"
            role="group"
            label={props.position > 0 ? t("racing.lap.positionValue", { position: props.position, racers: props.racers }) : t("racing.lap.position")}
          >
            <Text class={`text-8xl leading-flat italic font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg compact:text-6xl`}>
              {props.position > 0 ? formatNumber(props.position) : t("racing.lap.pending")}
            </Text>
            <Text class={`text-4xl leading-flat italic font-bold whitespace-nowrap text-fg-muted text-stroke-3 text-stroke-bg compact:text-title`}>{t("racing.lap.of", { total: props.racers })}</Text>
          </View>
        </Show>
        <View class="flex min-w-0 flex-col items-start gap-y-2 pb-1 compact:gap-y-1 compact:pb-0">
          <Show when={props.laps > 0 || props.lap > 0}>
            <View
              class={`group/lap shrink-0 flex-row items-baseline gap-x-2 py-0.75 px-2.25 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}
              role="group"
              label={props.laps > 0 && props.lap > 0 ? t("racing.lap.lapValue", { lap: props.lap, laps: props.laps }) : t("racing.lap.lap")}
              data-final={props.finalLap}
            >
              <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted group-data-final/lap:text-hud-accent-ink`}>
                {props.finalLap ? t("racing.lap.finalLap") : t("racing.lap.lap")}
              </Text>
              <Text class={`text-reading font-bold tabular-nums whitespace-nowrap`}>{props.lap > 0 ? formatNumber(props.lap) : t("racing.lap.pending")}</Text>
              <Show when={props.laps > 0}>
                <Text class={`text-caption font-bold tabular-nums whitespace-nowrap text-fg-muted`}>{t("racing.lap.ofSpaced", { total: props.laps })}</Text>
              </Show>
            </View>
          </Show>
          <Show when={props.gained !== 0}>
            <View class={`group/gained flex-row items-center gap-x-1.25 max-w-full min-w-0 py-0.5 px-2 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`} data-trend-down={props.gained < 0}>
              <View
                class={`shrink-0 size-2 -rotate-45 border-l-regular border-b-regular border-hud-accent group-data-trend-down/gained:rotate-45 group-data-trend-down/gained:border-t-regular group-data-trend-down/gained:border-r-regular group-data-trend-down/gained:border-l-0 group-data-trend-down/gained:border-b-0 group-data-trend-down/gained:border-foe`}
              />
              <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-hud-accent-ink group-data-trend-down/gained:text-foe-ink compact:tracking-normal`}>
                {props.gained > 0 ? t("racing.lap.up", { n: props.gained }) : t("racing.lap.down", { n: -props.gained })}
              </Text>
            </View>
          </Show>
        </View>
      </View>
      {/* Times: on a plate, so the rows and times read over a bright scene in every pack. */}
      <View class={`flex-col gap-y-1 compact:gap-y-0.5 py-2 px-3 compact:py-1 compact:px-2 bg-glass nine-plate-cut ${packClass("Plate", "frame")}`} role="list" label={t("racing.lap.times")}>
        <View class="flex-row items-baseline justify-between gap-x-2 min-w-0" role="listitem">
          <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:tracking-normal`}>{t("racing.lap.current")}</Text>
          <Text class={`text-title leading-flat font-bold tabular-nums whitespace-nowrap text-fg`}>{props.current !== "" ? props.current : t("racing.lap.none")}</Text>
        </View>
        <View class="group/lastrow flex-row items-baseline justify-between gap-x-2 min-w-0 compact:hidden" role="listitem" data-fresh={props.freshBest}>
          <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:tracking-normal`}>{t("racing.lap.last")}</Text>
          <View class="flex-row items-baseline gap-x-1.5">
            <Show when={props.freshBest}>
              {/* A tag in the best lap's own colour (the fixed dark well and the violet ink): the pack's neutral chip does not dress it. */}
              <Text class={`shrink-0 py-0 px-1.25 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-epic bg-well nine-chip-cut`}>{t("racing.lap.newBest")}</Text>
            </Show>
            <Text class={`text-base font-bold tabular-nums whitespace-nowrap text-fg-muted`}>{props.last !== "" ? props.last : t("racing.lap.none")}</Text>
          </View>
        </View>
        <View class="flex-row items-baseline justify-between gap-x-2 min-w-0" role="listitem">
          <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:tracking-normal`}>{t("racing.lap.best")}</Text>
          <Text class={`shrink-0 py-0 px-1.5 text-base font-bold tabular-nums whitespace-nowrap text-epic bg-well nine-chip-cut`}>{props.best !== "" ? props.best : t("racing.lap.none")}</Text>
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
            <Text class={`min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:tracking-normal`}>{props.sector !== "" ? props.sector : t("racing.lap.split")}</Text>
            {/* A tag in the split's own colour ahead and behind (boon and foe fills with their inks); the pack's neutral chip dresses only the even state. */}
            <Text
              class={`shrink-0 py-0 px-1.5 text-reading font-bold tabular-nums whitespace-nowrap text-fg bg-well nine-chip-cut data-ahead:bg-boon data-ahead:nine-chip-cut data-ahead:text-boon-fg data-behind:bg-foe data-behind:nine-chip-cut data-behind:text-foe-fg ${(props.splitState ?? "even") === "even" ? packClass("Chip", "frame") : ""}`}
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
