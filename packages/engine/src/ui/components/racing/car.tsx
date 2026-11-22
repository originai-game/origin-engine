import { For, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingCarProps {
  /** The tyres in the game's order (front left, front right, rear left, rear right), keyed rows of { id, label, text, state }: label and text as the game words them (FL, 92); state good, worn or bad, a green, gold or red ink on the tag. Two to a line. Empty hides the tyres. */
  tyres?: ReadonlyArray<{ id: string; label: string; text: string; state: string }>;
  /** The fuel left, 0 to 1; negative hides the fuel. Default -1. */
  fuel?: number;
  /** The fuel as the game words it (18 L · 4 laps), after the bar; empty for none. */
  fuelText: string;
  /** The fuel is low: the bar turns to danger with a Fuel low word (data-low). */
  fuelLow: boolean;
  /** The team calls the car in: a Box this lap tag stands under the readout (data-pit). */
  pit: boolean;
}

/**
 * The car's condition at the bottom left: the tyres as tags two to a line,
 * each worded and inked by its wear, the fuel as a thin bar with the
 * game's words after it (danger and Fuel low when it runs low), and Box
 * this lap when the team calls the car in. Read only.
 */
export function RacingCar(props: RacingCarProps) {
  return (
    <View class={`group/car flex-col gap-y-1.5 w-fit max-w-full items-start text-fg compact:gap-y-0.75`} role="group" label={t("racing.car.name")} data-pit={props.pit} data-low={props.fuelLow}>
      <Show when={(props.tyres ?? []).length}>
        <Text class={`max-w-full text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted compact:hidden compact:tracking-normal py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>{t("racing.car.tyres")}</Text>
        <View class={`grid grid-cols-2 gap-1.25 max-w-full compact:gap-0.75`} role="list" label={t("racing.car.tyres")}>
          <For each={props.tyres ?? []} key={(y) => y.id}>
            {(y) => (
              <Text
                class={`min-w-0 py-0 px-1.5 text-xs leading-normal font-bold tabular-nums whitespace-nowrap overflow-hidden text-fg bg-well nine-chip-cut data-good:text-boon-ink data-worn:text-gold data-bad:text-foe-ink ${packClass("Chip", "frame")}`}
                role="listitem"
                data-good={y().state === "good"}
                data-worn={y().state === "worn"}
                data-bad={y().state === "bad"}
              >{`${y().label} ${y().text}`}</Text>
            )}
          </For>
        </View>
      </Show>
      <Show when={(props.fuel ?? -1) >= 0}>
        <View
          class={`flex-row items-center gap-x-2 max-w-full min-w-0`}
          role="meter"
          label={props.fuelLow ? t("racing.car.fuelLow") : t("racing.car.fuel")}
          valueMin={0}
          valueMax={100}
          valueNow={(props.fuel ?? 0) >= 1 ? 100 : (props.fuel ?? 0) * 100}
          valueText={props.fuelText}
        >
          <Text class={`shrink-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted group-data-low/car:text-foe-ink compact:tracking-normal py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>
            {props.fuelLow ? t("racing.car.fuelLow") : t("racing.car.fuel")}
          </Text>
          <View class={`shrink-0 relative w-17.5 h-1.5 overflow-hidden bg-well shadow-plate ring ring-rim group-data-low/car:shadow-glow group-data-low/car:shadow-foe group-data-low/car:ring group-data-low/car:ring-foe compact:w-11`}>
            <View class="absolute left-0 top-0 h-full bg-fg group-data-low/car:bg-foe group-data-low/car:shadow-glow group-data-low/car:shadow-foe" style={{ width: `${Math.min(props.fuel ?? 0, 1) * 100}%` }} />
          </View>
          <Show when={props.fuelText !== ""}>
            <Text class={`shrink min-w-0 text-xs font-bold tabular-nums whitespace-nowrap overflow-hidden text-fg py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>{props.fuelText}</Text>
          </Show>
        </View>
      </Show>
      <Show when={props.pit}>
        <View class={`max-w-full min-w-0 py-0 px-2 bg-gold nine-chip-cut compact:tracking-normal text-gold-fg ${packClass("Chip", "frame")}`} role="status">
          <Text class={`text-xs leading-loose font-bold tracking-label uppercase whitespace-nowrap overflow-hidden compact:tracking-normal`}>{t("racing.car.pit")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingCarProps>> = {
  default: { "tyres": [{ "id": "fl", "label": "FL", "text": "92", "state": "good" }, { "id": "fr", "label": "FR", "text": "90", "state": "good" }, { "id": "rl", "label": "RL", "text": "71", "state": "worn" }, { "id": "rr", "label": "RR", "text": "48", "state": "bad" }], "fuel": -1, "fuelText": "", "fuelLow": false, "pit": false },
  fuel: { "tyres": [{ "id": "fl", "label": "FL", "text": "88", "state": "good" }, { "id": "fr", "label": "FR", "text": "86", "state": "good" }, { "id": "rl", "label": "RL", "text": "80", "state": "good" }, { "id": "rr", "label": "RR", "text": "79", "state": "good" }], "fuel": 0.62, "fuelText": "38 L · 7 laps", "fuelLow": false, "pit": false },
  fuelLow: { "tyres": [{ "id": "fl", "label": "FL", "text": "64", "state": "worn" }, { "id": "fr", "label": "FR", "text": "61", "state": "worn" }, { "id": "rl", "label": "RL", "text": "33", "state": "bad" }, { "id": "rr", "label": "RR", "text": "29", "state": "bad" }], "fuel": 0.08, "fuelText": "4 L · 1 lap", "fuelLow": true, "pit": true },
  overflow: { "tyres": [{ "id": "fl", "label": "Front left", "text": "100 % · 104 °C", "state": "good" }, { "id": "fr", "label": "Front right", "text": "100 % · 104 °C", "state": "good" }, { "id": "rl", "label": "Rear left", "text": "3 % · 141 °C", "state": "bad" }, { "id": "rr", "label": "Rear right", "text": "0 % · puncture", "state": "bad" }], "fuel": 3, "fuelText": "112.5 litres · 23 laps at this pace", "fuelLow": true, "pit": true },
};
