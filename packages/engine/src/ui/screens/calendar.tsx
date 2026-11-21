import { For, Icon, Show, Text, type UiEvent, View, packClass } from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface CalendarProps {
  /** The month shown (March 2027). */
  month: string;
  /** The seven weekday headings, a keyed collection of { id, label }. */
  weekdays: ReadonlyArray<{ id: string; label: string }>;
  /** The cells, a keyed collection of { id, label, outside, today, marked } from the week's first day (outside: another month's). */
  days: ReadonlyArray<{ id: string; label: string; today: boolean; outside: boolean; marked: boolean }>;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The pick event, with the row's key where it comes from a row. */
  onPick?: (e: UiEvent, key?: string) => void;
  /** The previous event, with the row's key where it comes from a row. */
  onPrevious?: (e: UiEvent, key?: string) => void;
}

/**
 * A HUD plate: the month between cut arrow keys, the days dark recesses,
 * today lit in the accent and a marked day carrying a gold diamond.
 */
export function Calendar(props: CalendarProps) {
  return (
    <View class={`nine-panel-cut bg-plate-hi border border-rim text-fg w-full max-w-90 p-3 gap-2 ${packClass("Panel", "frame")}`} label={props.month}>
      <View class={`flex-row min-h-12 border-b border-rim-soft items-center py-1 px-4 gap-1`}>
        <Button class={`bg-tile inset-ring-1 inset-ring-rim-soft min-w-11 min-h-11 p-0 rounded-control ${packClass("Button", "plate")}`} onPress={(e) => props.onPrevious?.(e)} label={props.month}>
          <Icon class="size-4" name="chevronLeft" />
        </Button>
        <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg grow shrink basis-0 text-center m-0`}>
          {props.month}
        </Text>
        <Button class={`bg-tile inset-ring-1 inset-ring-rim-soft min-w-11 min-h-11 p-0 rounded-control ${packClass("Button", "plate")}`} onPress={(e) => props.onNext?.(e)} label={props.month}>
          <Icon class="size-4" name="chevronRight" />
        </Button>
      </View>
      <View class="grid grid-cols-7 gap-0.5" role="grid">
        <For each={props.weekdays}>
          {(w) => (
          <Text class={`tracking-label uppercase text-fg-muted text-center text-xs font-semibold pb-0.5`}>
            {w().label}
          </Text>
          )}
        </For>
        <For each={props.days}>
          {(d) => (
          <View class={`flex-row items-center justify-center gap-1 border-rim-soft bg-tile inset-ring-1 inset-ring-rim-soft text-fg transition duration-317 w-full min-h-11 font-bold border data-selected:border-accent data-done:border-success active:border-accent data-outside:bg-none data-outside:bg-transparent data-outside:shadow-none data-outside:text-fg-muted data-today:bg-linear-to-b data-today:from-hud-accent-hi data-today:to-hud-accent data-today:bg-transparent data-today:shadow-none data-today:text-hud-accent-fg active:scale-96 data-locked:scale-100 disabled:scale-100 data-down:opacity-45`} role="button" current={d().today} onPress={(e) => props.onPick?.(e, d().id)} data-outside={d().outside} data-marked={d().marked} data-today={d().today} data-variant-primary={(d().today ? "primary" : "quiet") === "primary"} data-variant-quiet={(d().today ? "primary" : "quiet") === "quiet"}>
            <Text class="tabular-nums">{d().label}</Text>
            <Show when={d().marked}>
              <Text class="absolute bottom-0.5 rotate-45 bg-gold shadow-glow-sm shadow-gold tabular-nums size-1" />
            </Show>
          </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CalendarProps>> = {
  default: {"month":"March 2027","weekdays":[{"id":"1","label":"M"},{"id":"2","label":"T"},{"id":"3","label":"W"},{"id":"4","label":"T"},{"id":"5","label":"F"},{"id":"6","label":"S"},{"id":"7","label":"S"}],"days":[{"id":"2027-03-01","label":"1","today":false,"outside":false,"marked":false},{"id":"2027-03-02","label":"2","today":false,"outside":false,"marked":false},{"id":"2027-03-03","label":"3","today":false,"outside":false,"marked":false},{"id":"2027-03-04","label":"4","today":false,"outside":false,"marked":true},{"id":"2027-03-05","label":"5","today":false,"outside":false,"marked":false},{"id":"2027-03-06","label":"6","today":false,"outside":false,"marked":false},{"id":"2027-03-07","label":"7","today":false,"outside":false,"marked":false},{"id":"2027-03-08","label":"8","today":false,"outside":false,"marked":false},{"id":"2027-03-09","label":"9","today":false,"outside":false,"marked":false},{"id":"2027-03-10","label":"10","today":false,"outside":false,"marked":false},{"id":"2027-03-11","label":"11","today":false,"outside":false,"marked":true},{"id":"2027-03-12","label":"12","today":false,"outside":false,"marked":false},{"id":"2027-03-13","label":"13","today":false,"outside":false,"marked":false},{"id":"2027-03-14","label":"14","today":false,"outside":false,"marked":false},{"id":"2027-03-15","label":"15","today":false,"outside":false,"marked":false},{"id":"2027-03-16","label":"16","today":true,"outside":false,"marked":false},{"id":"2027-03-17","label":"17","today":false,"outside":false,"marked":false},{"id":"2027-03-18","label":"18","today":false,"outside":false,"marked":false},{"id":"2027-03-19","label":"19","today":false,"outside":false,"marked":false},{"id":"2027-03-20","label":"20","today":false,"outside":false,"marked":false},{"id":"2027-03-21","label":"21","today":false,"outside":false,"marked":false},{"id":"2027-03-22","label":"22","today":false,"outside":false,"marked":false},{"id":"2027-03-23","label":"23","today":false,"outside":false,"marked":false},{"id":"2027-03-24","label":"24","today":false,"outside":false,"marked":false},{"id":"2027-03-25","label":"25","today":false,"outside":false,"marked":true},{"id":"2027-03-26","label":"26","today":false,"outside":false,"marked":false},{"id":"2027-03-27","label":"27","today":false,"outside":false,"marked":false},{"id":"2027-03-28","label":"28","today":false,"outside":false,"marked":false},{"id":"2027-03-29","label":"29","today":false,"outside":false,"marked":false},{"id":"2027-03-30","label":"30","today":false,"outside":false,"marked":false},{"id":"2027-03-31","label":"31","today":false,"outside":false,"marked":false},{"id":"2027-04-01","label":"1","today":false,"outside":true,"marked":false},{"id":"2027-04-02","label":"2","today":false,"outside":true,"marked":false},{"id":"2027-04-03","label":"3","today":false,"outside":true,"marked":false},{"id":"2027-04-04","label":"4","today":false,"outside":true,"marked":false}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"previous":"Previous month","next":"Next month","pick":"Choose day"} as const;
