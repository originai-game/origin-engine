import { For, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

/** The rev arc's start: the lower left, −135° from up, sweeping clockwise over the top. */
const REV_START = -(3 * Math.PI) / 4;

export interface RacingTelemetryProps {
  /** The speed as the game measures it, already in its units (a whole number reads best). */
  speed: number;
  /** The speed's unit as the game words it (km/h, mph); shown under the figure and read with it. */
  units: string;
  /** The gear as the game names it: R, N, 1 to 8; on a tag in the HUD's accent; empty hides it. */
  gear: string;
  /** The gearbox's mode as the game words it (Auto, Manual), beside the gear; empty hides it. */
  mode: string;
  /** Engine speed now: the rev arc's length. */
  rpm: number;
  /** The top of the rev range, the arc's end. Default 1. */
  rpmMax?: number;
  /** The engine speed where the red zone starts, marked on the arc (lit, with data-redline on the frame, once the revs reach it); 0 is none. */
  redline: number;
  /** The dial's figures, keyed rows of { id, value, label }: value an engine speed on the arc, label its figure as the game words it (2 for 2,000 rpm); a figure at or past the redline is red. A tick stands by each. Empty draws none. A phone shows the ticks alone. */
  scale?: ReadonlyArray<{ id: string; value: number; label: string }>;
  /** The share of the shift lights lit, 0 to 1, as the game decides it (seven cells, green to amber to red); negative hides the lights. Default -1. */
  lights?: number;
  /** The game's shift cue: the Shift word stands beside the gear in place of the mode, and the frame takes data-shift. */
  shift: boolean;
}

/**
 * The tachometer at the bottom right: the revs as a white arc round a dark
 * glass face with the red zone at its end, the dial's figures inside the
 * arc, and on the face the shift lights, the speed in the game's units and
 * the gear on a lit tag with the gearbox's mode. The game decides every
 * figure, the lights and the shift cue; read only.
 */
export function RacingTelemetry(props: RacingTelemetryProps) {
  const max = () => props.rpmMax ?? 1;
  const at = (v: number) => (max() > 0 ? (v <= 0 ? 0 : v >= max() ? 1 : v / max()) : 0);
  const revFrac = () => (max() > 0 ? (props.rpm >= max() ? 0.75 : props.rpm <= 0 ? 0 : (0.75 * props.rpm) / max()) : 0);
  const angle = (v: number) => (at(v) * 270 - 135) * (Math.PI / 180);
  return (
    <View
      class={`group/tel w-fit max-w-full text-fg`}
      role="group"
      label={t("racing.telemetry.name")}
      data-shift={props.shift}
      data-redline={props.redline > 0 && props.rpm >= props.redline}
    >
      <View
        class={`relative size-[16.429rem] max-w-full compact:size-[10.571rem]`}
        role="meter"
        label={t("racing.telemetry.rpm")}
        valueMin={0}
        valueMax={max()}
        valueNow={props.rpm}
        valueText={t("racing.telemetry.rpmValue", { rpm: props.rpm })}
      >
        {/* The dial (svg arcs in the svelte): the well ring, then the revs as a nine-ring wipe from the lower left clockwise over the top (the arc spans 270°, so the wipe is the rev share of 0.75), then the red zone as a second nine-ring over its own span. */}
        <View class="absolute inset-0 rounded-full border-[8px] border-white/16" />
        <View class="absolute inset-0 nine-ring bg-white/85" wipe={revFrac()} wipeStart={REV_START} />
        <Show when={props.redline > 0 && max() > 0 && props.redline < max()}>
          <View
            class="absolute inset-0 nine-ring bg-critical/35 group-data-redline/tel:bg-critical"
            wipe={0.75 - 0.75 * at(props.redline)}
            wipeStart={REV_START + 1.5 * Math.PI * at(props.redline)}
          />
        </Show>
        <For each={props.scale ?? []} key={(s) => s.id}>
          {(s) => {
            const a = () => angle(s().value);
            const deg = () => at(s().value) * 270 - 135;
            const red = () => props.redline > 0 && s().value >= props.redline;
            return (
              <>
                <View
                  class={`absolute w-[0.143rem] h-[0.5rem] bg-white data-red:bg-foe compact:h-[0.357rem]`}
                  style={{ left: `${50 + Math.sin(a()) * 40.5}%`, top: `${50 - Math.cos(a()) * 40.5}%`, transform: `translate(-50%,-50%) rotate(${deg()}deg)` }}
                  data-red={red()}
                />
                <Text
                  class={`absolute text-[0.857rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg data-red:text-foe compact:hidden`}
                  style={{ left: `${50 + Math.sin(a()) * 35}%`, top: `${50 - Math.cos(a()) * 35}%`, transform: "translate(-50%,-50%)" }}
                  data-red={red()}
                >
                  {s().label}
                </Text>
              </>
            );
          }}
        </For>
        <View
          class={`absolute inset-0 m-auto w-[60%] aspect-square flex-col items-center justify-center gap-y-[0.214rem] p-[0.429rem] rounded-full bg-glass shadow-[inset_0px_1px_0px_#ffffff4d,0px_10px_30px_#00000099] compact:gap-y-[0.071rem] compact:p-[0.214rem]`}
          role="group"
          label={t("racing.telemetry.speed", { speed: props.speed, units: props.units })}
        >
          <Show when={(props.lights ?? -1) >= 0}>
            <View
              class={`w-[40%] h-[0.429rem] mb-[0.143rem] overflow-hidden bg-white/22 compact:h-[0.286rem]`}
              role="meter"
              label={t("racing.telemetry.lights")}
              valueMin={0}
              valueMax={100}
              valueNow={(props.lights ?? 0) >= 1 ? 100 : (props.lights ?? 0) * 100}
            >
              <View
                class="absolute left-0 top-0 h-full bg-linear-to-r from-boon from-[43%] via-gold via-[72%] to-foe"
                style={{ width: `${Math.min(Math.max(props.lights ?? 0, 0), 1) * 100}%` }}
              />
            </View>
          </Show>
          <Text class={`text-[4.571rem] leading-[0.82] font-bold tabular-nums whitespace-nowrap text-fg compact:text-[2.571rem] compact:leading-none`}>{formatNumber(props.speed)}</Text>
          <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg/78 compact:tracking-normal compact:leading-none`}>{props.units}</Text>
          <Show when={props.gear !== "" || props.mode !== "" || props.shift}>
            <View class={`flex-row items-center justify-center gap-x-[0.571rem] max-w-full min-w-0 mt-[0.286rem] compact:absolute compact:top-full compact:left-0 compact:right-0 compact:mt-[0.143rem] compact:gap-x-[0.357rem] compact:justify-start`}>
              <Show when={props.gear !== ""}>
                <Text
                  class={`shrink-0 py-0 px-2 text-[1.143rem] leading-[1.2] font-bold tabular-nums whitespace-nowrap text-gold-fg bg-hud-accent nine-chip-cut compact:text-[0.929rem] ${packClass("Chip", "frame")}`}
                  role="image"
                  label={t("racing.telemetry.gearValue", { gear: props.gear })}
                >
                  {props.gear}
                </Text>
              </Show>
              <Show when={props.shift} fallback={
                <Show when={props.mode !== ""}>
                  <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg-muted compact:hidden compact:tracking-normal compact:leading-none`}>{props.mode}</Text>
                </Show>
              }>
                <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-foe compact:tracking-normal compact:leading-none`} role="status">{t("racing.telemetry.shift")}</Text>
              </Show>
            </View>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingTelemetryProps>> = {
  default: { "speed": 247, "units": "km/h", "gear": "5", "mode": "Auto", "rpm": 6900, "rpmMax": 10500, "redline": 8500, "lights": 0.857, "shift": false, "scale": [{ "id": "0", "value": 0, "label": "0" }, { "id": "2", "value": 2000, "label": "2" }, { "id": "4", "value": 4000, "label": "4" }, { "id": "6", "value": 6000, "label": "6" }, { "id": "8", "value": 8000, "label": "8" }, { "id": "10", "value": 10000, "label": "10" }] },
  redline: { "speed": 288, "units": "km/h", "gear": "6", "mode": "Manual", "rpm": 9800, "rpmMax": 10500, "redline": 8500, "lights": 1, "shift": true, "scale": [{ "id": "0", "value": 0, "label": "0" }, { "id": "2", "value": 2000, "label": "2" }, { "id": "4", "value": 4000, "label": "4" }, { "id": "6", "value": 6000, "label": "6" }, { "id": "8", "value": 8000, "label": "8" }, { "id": "10", "value": 10000, "label": "10" }] },
  pit: { "speed": 60, "units": "km/h", "gear": "2", "mode": "Limiter", "rpm": 3100, "rpmMax": 10500, "redline": 8500, "lights": 0, "shift": false, "scale": [{ "id": "0", "value": 0, "label": "0" }, { "id": "2", "value": 2000, "label": "2" }, { "id": "4", "value": 4000, "label": "4" }, { "id": "6", "value": 6000, "label": "6" }, { "id": "8", "value": 8000, "label": "8" }, { "id": "10", "value": 10000, "label": "10" }] },
  reverse: { "speed": 12, "units": "mph", "gear": "R", "mode": "", "rpm": 2300, "rpmMax": 8000, "redline": 7000, "lights": -1, "shift": false, "scale": [] },
  idle: { "speed": 0, "units": "km/h", "gear": "N", "mode": "Auto", "rpm": 900, "rpmMax": 9000, "redline": 0, "lights": 0, "shift": false, "scale": [{ "id": "0", "value": 0, "label": "0" }, { "id": "3", "value": 3000, "label": "3" }, { "id": "6", "value": 6000, "label": "6" }, { "id": "9", "value": 9000, "label": "9" }] },
  overflow: { "speed": 1408, "units": "Kilometer pro Stunde", "gear": "10", "mode": "Sequenziell mit Wippen", "rpm": 18500, "rpmMax": 18000, "redline": 15000, "lights": 1, "shift": true, "scale": [{ "id": "0", "value": 0, "label": "0" }, { "id": "6", "value": 6000, "label": "6000" }, { "id": "12", "value": 12000, "label": "12000" }, { "id": "18", "value": 18000, "label": "18000" }] },
};
