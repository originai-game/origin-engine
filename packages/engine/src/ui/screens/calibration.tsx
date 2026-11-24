import {Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./calibration.en.ogt";

/** A slider stands in for the Svelte range input, which the TSX host has no surface for: a track, a fill and a thumb stepped by the arrow keys (a reader's increment and decrement arrive as ArrowUp and ArrowDown). Pointer dragging has no surface; the game owns the value and each step reports it back. */
function Slider(props: {
  value: number;
  min: number;
  max: number;
  step: number;
  label: string;
  onChange?: (e: UiEvent, value: number) => void;
}) {
  const clamped = () => Math.min(props.max, Math.max(props.min, props.value));
  const percent = () => ((clamped() - props.min) / (props.max - props.min || 1)) * 100;
  const stepped = (e: UiEvent, up: boolean) => props.onChange?.(e, Math.min(props.max, Math.max(props.min, clamped() + (up ? props.step : -props.step))));
  /** The pointer's drag: the track's box rides in e.value ("x y width height"), pointerdown captures the moves. */
  const dragged = (e: UiEvent) => {
    const [bx, , bw] = e.value.split(" ").map(Number);
    const raw = props.min + Math.min(1, Math.max(0, (e.x - bx) / (bw || 1))) * (props.max - props.min);
    props.onChange?.(e, Math.min(props.max, Math.max(props.min, props.min + Math.round((raw - props.min) / props.step) * props.step)));
  };
  return (
    <View
      class="grow shrink basis-50 min-w-0 flex-row items-center h-9"
      role="slider"
      tabIndex={0}
      label={props.label}
      valueMin={props.min}
      valueMax={props.max}
      valueNow={clamped()}
      onKey={(e) => {
        if (e.key === "ArrowRight" || e.key === "ArrowUp") stepped(e, true);
        else if (e.key === "ArrowLeft" || e.key === "ArrowDown") stepped(e, false);
      }}
    >
      <View class="relative grow shrink basis-0 h-1 rounded-capsule bg-track inset-shadow-lit ring ring-rim" onPointerDown={dragged} onPointerMove={dragged}>
        <View class="absolute left-0 top-0 h-full rounded-capsule bg-accent" style={{ width: `${percent()}%` }} />
        <View class="absolute top-0 size-2.5 -translate-x-1/2 -translate-y-1 rounded-capsule bg-accent shadow-panel" style={{ left: `${percent()}%` }} />
      </View>
    </View>
  );
}

export interface CalibrationScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** brightness, offset or controller. */
  step: string;
  /** The gamma's adjustment, 0 to 1 (two-way). */
  brightness: number;
  /** The audio offset in ms, -200 to 200 (two-way). */
  offset: number;
  /** Taps measured so far. */
  taps: number;
  /** The sticks' dead zone, 0 to 0.5 (two-way). */
  deadZone: number;
  /** The brightness slider stepped (onBrightness), the value clamped to 0 to 1. */
  onBrightness?: (e: UiEvent, value: number) => void;
  /** The offset slider stepped (onOffset), the value clamped to -200 to 200 ms. */
  onOffset?: (e: UiEvent, value: number) => void;
  /** The dead-zone slider stepped (onDeadZone), the value clamped to 0 to 0.5. */
  onDeadZone?: (e: UiEvent, value: number) => void;
  /** Content the caller supplies (was the test snippet). */
  test?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The done event, with the row's key where it comes from a row. */
  onDone?: (e: UiEvent, key?: string) => void;
  /** The measure event, with the row's key where it comes from a row. */
  onMeasure?: (e: UiEvent, key?: string) => void;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The tap event, with the row's key where it comes from a row. */
  onTap?: (e: UiEvent, key?: string) => void;
}

/**
 * Calibration in three steps: brightness (gamma), the audio offset measured
 * from taps, and the controller's dead zone, each with its slider and test.
 * The game owns the values and writes them back from the events.
 */
export function CalibrationScreen(props: CalibrationScreenProps) {
  return (
    <View class={`absolute inset-0 grow shrink basis-0 justify-center items-center text-fg w-full p-8 gap-2 compact:p-3 ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <View class="absolute inset-0" />
      <View class={`nine-panel-cut bg-layer-1 text-fg gap-y-4 w-full max-w-190 py-6 px-8 compact:gap-y-3 compact:p-4 disabled:text-fg-subtle ${packClass("Panel", "frame")}`}>
        <View class="flex-row items-center gap-x-2 p-0 m-0 compact:gap-x-1">
          <View class={`group/brightness flex-row items-center gap-x-2 text-caption font-semibold tracking-label uppercase text-fg-muted compact:gap-x-1 compact:tracking-md data-done:text-fg-muted data-on:text-fg compact:data-on:grow-0 compact:data-on:shrink compact:data-on:basis-auto compact:data-on:min-w-0`} data-on={props.step === "brightness"} data-done={props.step !== "brightness"}>
            <View class="grid items-center justify-items-center inset-ring-hairline inset-ring-rim text-caption size-6.5 rounded-full group-data-on/brightness:shadow-glow group-data-on/brightness:bg-none group-data-on/brightness:bg-hud-accent group-data-on/brightness:text-hud-accent-fg group-data-done/brightness:bg-none group-data-done/brightness:bg-raised-hi shrink">
              <Text class="tabular-nums whitespace-nowrap">{1}</Text>
            </View>
            <Text class="whitespace-nowrap m-0 compact:hidden compact:group-data-on/brightness:flex compact:group-data-on/brightness:min-w-0 compact:group-data-on/brightness:text-ellipsis compact:group-data-on/brightness:overflow-hidden shrink">
              {t("screen.calibration.brightness")}
            </Text>
          </View>
          <View class="grow shrink basis-0 min-w-2 h-px bg-rim compact:min-w-1" role="separator" />
          <View class={`group/offset flex-row items-center gap-x-2 text-caption font-semibold tracking-label uppercase text-fg-muted compact:gap-x-1 compact:tracking-md data-done:text-fg-muted data-on:text-fg compact:data-on:grow-0 compact:data-on:shrink compact:data-on:basis-auto compact:data-on:min-w-0`} data-on={props.step === "offset"} data-done={props.step === "controller"}>
            <View class="grid items-center justify-items-center inset-ring-hairline inset-ring-rim text-caption size-6.5 rounded-full group-data-on/offset:shadow-glow group-data-on/offset:bg-none group-data-on/offset:bg-hud-accent group-data-on/offset:text-hud-accent-fg group-data-done/offset:bg-none group-data-done/offset:bg-raised-hi shrink">
              <Text class="tabular-nums whitespace-nowrap">{2}</Text>
            </View>
            <Text class="whitespace-nowrap m-0 compact:hidden compact:group-data-on/offset:flex compact:group-data-on/offset:min-w-0 compact:group-data-on/offset:text-ellipsis compact:group-data-on/offset:overflow-hidden shrink">
              {t("screen.calibration.offset")}
            </Text>
          </View>
          <View class="grow shrink basis-0 min-w-2 h-px bg-rim compact:min-w-1" role="separator" />
          <View class={`group/controller flex-row items-center gap-x-2 text-caption font-semibold tracking-label uppercase text-fg-muted compact:gap-x-1 compact:tracking-md data-on:text-fg compact:data-on:grow-0 compact:data-on:shrink compact:data-on:basis-auto compact:data-on:min-w-0`} data-on={props.step === "controller"}>
            <View class="grid items-center justify-items-center inset-ring-hairline inset-ring-rim text-caption size-6.5 rounded-full group-data-on/controller:shadow-glow group-data-on/controller:bg-none group-data-on/controller:bg-hud-accent group-data-on/controller:text-hud-accent-fg shrink">
              <Text class="tabular-nums whitespace-nowrap">{3}</Text>
            </View>
            <Text class="whitespace-nowrap m-0 compact:hidden compact:group-data-on/controller:flex compact:group-data-on/controller:min-w-0 compact:group-data-on/controller:text-ellipsis compact:group-data-on/controller:overflow-hidden shrink">
              {t("screen.calibration.controller")}
            </Text>
          </View>
        </View>
        <View class={`min-h-40 inset-ring-hairline inset-ring-rim compact:min-h-32`}>
          {props.test?.()}
        </View>
        <Show when={props.step === "brightness"} fallback={
            <Show when={props.step === "offset"} fallback={
                <Show when={props.step === "controller"}>
                  <View class="gap-y-3">
                    <Text class="text-base leading-normal text-fg-muted line-clamp-3 m-0 compact:text-base">
                      {t("screen.calibration.controllerHint")}
                    </Text>
                    <View class="flex-row flex-wrap items-center gap-y-2 gap-x-3">
                      <Text class={`tracking-xs text-fg-muted text-base leading-normal line-clamp-3 font-semibold m-0 compact:text-base shrink`}>
                        {t("screen.calibration.deadZone")}
                      </Text>
                      <Slider label={t("screen.calibration.deadZone")} value={props.deadZone} min={0} max={0.5} step={0.01} onChange={(e, v) => props.onDeadZone?.(e, v)} />
                    </View>
                  </View>
                </Show>
            }>
              <View class="gap-y-3">
                <Text class="text-base leading-normal text-fg-muted line-clamp-3 m-0 compact:text-base">
                  {t("screen.calibration.offsetHint")}
                </Text>
                <View class="flex-row flex-wrap items-center gap-y-2 gap-x-3">
                  <Button class={`text-fg bg-layer-3 nine-plate-cut min-w-40 h-9 disabled:border-transparent active:border-transparent hover:bg-plate-hi hover:bg-transparent hover:border-transparent compact:min-w-0 compact:grow compact:shrink compact:basis-full data-pressed:hover:border-transparent`} onPress={(e) => props.onMeasure?.(e)}>
                    <Text>{t("screen.calibration.start")}</Text>
                  </Button>
                  <Button class={`text-accent nine-plate-cut min-w-40 h-9 compact:min-w-0 compact:grow compact:shrink compact:basis-0`} onPress={(e) => props.onTap?.(e)}>
                    <Text>{t("screen.calibration.tap")}</Text>
                  </Button>
                  <Text class="text-base leading-normal text-fg-muted line-clamp-3 tabular-nums m-0 compact:text-base shrink">
                    {t("screen.calibration.Taps", { taps: props.taps })}
                  </Text>
                </View>
                <View class="flex-row flex-wrap items-center gap-y-2 gap-x-3">
                  <Slider label={t("screen.calibration.offset")} value={props.offset} min={-200} max={200} step={1} onChange={(e, v) => props.onOffset?.(e, v)} />
                  <Text class="min-w-16 text-end text-lg font-semibold tabular-nums shrink">
                    {t("screen.calibration.Ms", { offset: props.offset })}
                  </Text>
                </View>
              </View>
            </Show>
        }>
          <View class="gap-y-3">
            <Text class="text-base leading-normal text-fg-muted line-clamp-3 m-0 compact:text-base">
              {t("screen.calibration.brightnessHint")}
            </Text>
            <Slider label={t("screen.calibration.brightness")} value={props.brightness} min={0} max={1} step={0.01} onChange={(e, v) => props.onBrightness?.(e, v)} />
          </View>
        </Show>
        <View class="flex-row flex-wrap items-center justify-between gap-2">
          <Button class={`text-fg bg-layer-3 nine-plate-cut min-w-40 h-9 disabled:border-transparent active:border-transparent hover:bg-plate-hi hover:bg-transparent hover:border-transparent compact:min-w-0 compact:grow compact:shrink compact:basis-0 data-pressed:hover:border-transparent`} onPress={(e) => props.onBack?.(e)}>
            <Text>{t("screen.common.back")}</Text>
          </Button>
          <Show when={props.step === "controller"} fallback={
              <Button variant="primary" class={`nine-plate-cut min-w-40 h-9 compact:min-w-0 compact:grow compact:shrink compact:basis-0`} onPress={(e) => props.onNext?.(e)}>
                <Text>{t("screen.calibration.next")}</Text>
              </Button>
          }>
            <Button variant="primary" class={`nine-plate-cut min-w-40 h-9 compact:min-w-0 compact:grow compact:shrink compact:basis-0`} onPress={(e) => props.onDone?.(e)}>
              <Text>{t("screen.calibration.done")}</Text>
            </Button>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CalibrationScreenProps>> = {
  default: {"brightness":0.5,"deadZone":0.12,"offset":24,"step":"offset","taps":6},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Next","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"calibration","layer":"menu","focus":"Panel/Foot/Next","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
