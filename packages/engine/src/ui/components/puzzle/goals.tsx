import {For, Icon, Show, Text, View, formatDuration, formatNumber, t, packClass} from "@origingame/engine/ui";
import "./puzzle.en.ogt";

export interface PuzzleGoalsProps {
  /** The level's goals in order, keyed rows of { id, icon, label, left, done }: left how many are still to clear, as the game counts them; done the goal is met (a check replaces the count). */
  goals?: ReadonlyArray<{ id: string; icon: string; label: string; left: number; done: boolean }>;
  /** What limits the level: moves (a move count) or time (a countdown). */
  counter: string;
  /** Moves left, when counter is moves. */
  moves: number;
  /** Seconds left, when counter is time. */
  seconds: number;
  /** The game says the moves or time are running low: the counter warns, in words as well as its dress. */
  low?: boolean;
  /** The score so far, as the game counts it. */
  score: number;
  /** The score toward the level's top star, 0 to 1, for the bar under it; negative hides the bar. Default -1. */
  progress?: number;
  /** The star marks on that bar, keyed rows of { id, at }: at where the star sits along the bar, 0 to 1; a star at or under progress is lit. */
  stars?: ReadonlyArray<{ id: string; at: number }>;
}

/**
 * The in-level HUD: what limits the level (moves or time left) in a cut
 * well, amber with a tag in words when the game says it runs low; the goals
 * as sockets with what is left of each, its name under it and a check once
 * met; and the score over a leaning bar with the game's star marks, lit up
 * to its progress. It draws what the game counts; it decides nothing. Where
 * it is given less than a phone's width (its own, not the screen's) the
 * moves and the score share the top line and the goals wrap under them.
 */
export function PuzzleGoals(props: PuzzleGoalsProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-plate-hi to-plate-lo text-fg h-auto p-2 gap-2 rounded-block border disabled:text-fg-subtle`} label={t("puzzle.goals.name")} data-low={props.low === true}>
      <View class="flex-row flex-wrap p-1 gap-3 compact:gap-1">
        <View class={`items-center justify-center min-w-[102.4px] bg-tile border border-white/14 py-1 px-2 gap-0 compact:min-w-[83.2px] data-low:border-[1.5px] data-low:border-warn ${packClass("Plate", "frame")}`} role={props.counter === 'time' ? 'timer' : 'status'} label={props.counter === 'time' ? t("puzzle.goals.timeLeft", { time: formatDuration(props.seconds) }) : t("puzzle.goals.movesLeft", { moves: props.moves })} data-low={props.low === true}>
          <Text class={`tabular-nums text-fg whitespace-nowrap text-[2.857rem] font-bold data-low:text-warn compact:text-[2.143rem]`} data-low={props.low === true}>
            {props.counter === 'time' ? formatDuration(props.seconds) : formatNumber(props.moves)}
          </Text>
          <Text class={`tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap text-[0.857rem] font-semibold text-shadow-2xs`}>
            {props.counter === 'time' ? t("puzzle.goals.time") : t("puzzle.goals.moves")}
          </Text>
          <Show when={props.low}>
            <Text class={`mt-0.5 whitespace-nowrap text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-bg bg-warn nine-chip-cut text-shadow-2xs py-0 px-1 ${packClass("Chip", "frame")}`}>
              {t("puzzle.goals.low")}
            </Text>
          </Show>
        </View>
        <View class={`flex-row flex-wrap grow shrink basis-0 justify-center gap-1 compact:basis-full`} role="list" label={t("puzzle.goals.name")}>
          <Show when={(props.goals)?.length ?? 0} fallback={<Text class={`tracking-xs text-fg-muted self-center text-center text-shadow-2xs m-0 shrink`}>   {t("puzzle.goals.none")} </Text>}><For each={props.goals}>
            {(g) => (
            <View class={`items-center min-w-[83.2px] max-w-32 bg-white/4 shadow-[inset_0px_0px_0px_1px_#ffffff14] py-0.5 px-1 gap-0 compact:min-w-0 compact:grow compact:shrink compact:basis-0`} role="listitem" label={g().done ? t("puzzle.goals.done", { label: g().label }) : t("puzzle.goals.left", { label: g().label, left: g().left })} data-done={g().done}>
              <View class={`group/socket grow shrink basis-0 justify-center items-center gap-2 size-12 compact:size-8`}>
                <Icon class="text-shadow-2xs size-full" name={g().icon} />
              </View>
              <Show when={g().done}>
                <Icon class={`absolute top-0.5 right-0.5 bg-success shadow-[0px_0px_9.136px_#45e08f] size-4 rounded-control text-bg`} name="check" />
              </Show>
              <Text class={`whitespace-nowrap text-[1.429rem] font-bold tabular-nums text-fg text-shadow-2xs data-done:text-success`} data-done={g().done}>
                {formatNumber(g().left)}
              </Text>
              <Text class={`text-fg/78 max-w-full whitespace-nowrap text-ellipsis text-[0.857rem] font-semibold overflow-hidden`}>
                {g().label}
              </Text>
            </View>
            )}
          </For></Show>
        </View>
        <View class={`justify-center min-w-[230.4px] py-1 px-2 gap-0.5 compact:grow compact:shrink compact:basis-0 compact:min-w-0`} role="status" label={t("puzzle.goals.scoreName", { score: formatNumber(props.score) })}>
          <View class="flex-row items-baseline gap-1">
            <Text class={`grow shrink basis-0 tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap text-[0.857rem] font-semibold text-shadow-2xs`}>
              {t("puzzle.goals.score")}
            </Text>
            <Text class={`tabular-nums text-fg whitespace-nowrap text-[1.714rem] font-bold shrink`}>
              {formatNumber(props.score)}
            </Text>
          </View>
          <Show when={(props.progress ?? -1) >= 0}>
            <View class="w-full h-4">
              <View class={`bg-tile shadow-sunk absolute top-[30%] h-[0.571rem] inset-x-0 rounded-capsule overflow-hidden`}>
                <View class={`bg-gold h-full`} style={{ width: `${(100 * ((props.progress ?? -1) > 1 ? 1 : (props.progress ?? -1)))}%` }} />
              </View>
              <For each={props.stars}>
                {(s) => (
                <View class={`absolute top-0 text-white/35 size-4 data-lit:text-gold`} data-lit={(props.progress ?? -1) >= s().at}>
                  <Icon class="text-shadow-2xs size-full" name="star" />
                </View>
                )}
              </For>
            </View>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<PuzzleGoalsProps>> = {
  default: {"progress":0.62,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":18,"seconds":0,"low":false,"score":12450,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":12,"done":false},{"id":"jelly","icon":"drop","label":"Jelly","left":4,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]},
  time: {"progress":0.2,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"time","moves":0,"seconds":74,"low":false,"score":3200,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":20,"done":false}]},
  low: {"progress":0.45,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":3,"seconds":0,"low":true,"score":8800,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":5,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]},
  lowTime: {"progress":0.75,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"time","moves":0,"seconds":7,"low":true,"score":15000,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":2,"done":false}]},
  done: {"progress":1,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":6,"seconds":0,"low":false,"score":24300,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":0,"done":true},{"id":"jelly","icon":"drop","label":"Jelly","left":0,"done":true}]},
  empty: {"progress":-1,"stars":[],"counter":"moves","moves":25,"seconds":0,"low":false,"score":0,"goals":[]},
  overflow: {"progress":1,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":1250,"seconds":0,"low":false,"score":123456789,"goals":[{"id":"a","icon":"apple","label":"Red sweets","left":1250,"done":false},{"id":"b","icon":"drop","label":"Jelly","left":99,"done":false},{"id":"c","icon":"leaf","label":"Leaves","left":0,"done":true},{"id":"d","icon":"gem","label":"Gems","left":340,"done":false},{"id":"e","icon":"star","label":"Stars","left":7,"done":false},{"id":"f","icon":"key","label":"Keys","left":3,"done":false}]},
};
