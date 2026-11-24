import { For, Show, Text, View, t, packClass } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmGaugeProps {
  /** What it measures: life (the run fails at empty) or fever (a charge the player spends when full); on data-kind. Default "life". */
  kind?: string;
  /** How full, 0 to 1. */
  value: number;
  /** The screen edge it stands at, start or end: it leans toward the play field. Default "start". */
  side?: string;
  /** The game says life is low: the gauge turns to danger, its rim pulses and Danger shows under it. */
  danger: boolean;
  /** The game says the run is failed (life gone): Failed shows under it. */
  failed: boolean;
  /** The game says the charge is ready to spend: the gauge glows and Ready shows. */
  ready: boolean;
  /** The input that spends it as the game names it (Space, RB); empty hides the key cap. */
  key: string;
}

/**
 * A tall gauge at a screen edge, cut into ten cells and leaning toward the
 * play field: the run's life (green; danger red with a pulsing rim under
 * the game's mark, Danger and Failed in words) or its fever charge (pink;
 * Ready and the key that spends it). Its height is the space the page
 * gives it. The game owns the value and every verdict; read only.
 */
export function RhythmGauge(props: RhythmGaugeProps) {
  const full = () => (props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value);
  return (
    <View
      class={`group/gauge flex-col items-center gap-1 h-full min-h-30 compact:min-h-0 max-w-24 w-full text-fg`}
      data-kind-fever={(props.kind ?? "life") === "fever"}
      data-side-end={(props.side ?? "start") === "end"}
      data-danger={props.danger}
      data-ready={props.ready}
    >
      <View
        class={`relative flex-1 min-h-0 w-6.5 compact:w-4 overflow-hidden -skew-y-12 group-data-side-end/gauge:skew-y-12 bg-well shadow-plate ring ring-rim data-danger:shadow-glow data-danger:shadow-foe data-danger:ring data-danger:ring-foe data-danger:animate-pulse data-ready:shadow-glow-lg data-ready:inset-ring-hairline data-ready:inset-ring-rim`}
        role="meter"
        label={(props.kind ?? "life") === "fever" ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")}
        valueMin={0}
        valueMax={100}
        valueNow={full() * 100}
        data-danger={props.danger}
        data-ready={props.ready}
      >
        <View
          class={`absolute bottom-0 left-0 w-full bg-boon shadow-glow-sm shadow-fill-strong group-data-kind-fever/gauge:bg-fever group-data-kind-fever/gauge:shadow-glow-sm group-data-kind-fever/gauge:shadow-fill-strong group-data-danger/gauge:bg-critical`}
          style={{ height: `${full() * 100}%` }}
        />
        <For each={Array.from({ length: 9 }, (_, i) => i)}>
          {(i) => <View class={`absolute left-0 right-0 h-0.75 -translate-y-1/2 bg-shade-deep`} style={{ top: `${(i() + 1) * 10}%` }} />}
        </For>
      </View>
      <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")} text-fg-muted group-data-kind-fever/gauge:text-fever-ink`}>
        {(props.kind ?? "life") === "fever" ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")}
      </Text>
      <Show when={props.failed} fallback={
        <Show when={props.danger}>
          <Text role="status" class={`text-xs font-bold tracking-label uppercase whitespace-nowrap text-danger text-stroke-3 text-stroke-bg`}>
            {t("rhythm.gauge.danger")}
          </Text>
        </Show>
      }>
        <Text role="status" class={`text-xs font-bold tracking-label uppercase whitespace-nowrap text-danger text-stroke-3 text-stroke-bg`}>
          {t("rhythm.gauge.failed")}
        </Text>
      </Show>
      <Show when={props.ready}>
        <Text role="status" class={`text-xs font-bold tracking-label uppercase whitespace-nowrap text-fever-ink text-stroke-3 text-stroke-bg`}>
          {t("rhythm.gauge.ready")}
        </Text>
      </Show>
      <Show when={props.key !== ""}>
        <Text
          class={`max-w-full px-1 text-center text-xs font-bold bg-linear-to-b from-key-hi to-key-lo text-hud-accent-fg rounded-control shadow-key data-ready:bg-linear-to-b data-ready:from-hud-accent-hi data-ready:to-hud-accent data-ready:shadow-glow-lg data-ready:inset-shadow-lip`}
          data-ready={props.ready}
        >
          {props.key}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmGaugeProps>> = {
  default: {"kind":"life","value":0.78,"side":"start","danger":false,"failed":false,"ready":false,"key":""},
  fever: {"kind":"fever","value":0.92,"side":"end","danger":false,"failed":false,"ready":false,"key":"Space"},
  ready: {"kind":"fever","value":1,"side":"end","danger":false,"failed":false,"ready":true,"key":"Space"},
  danger: {"kind":"life","value":0.14,"side":"start","danger":true,"failed":false,"ready":false,"key":""},
  failed: {"kind":"life","value":0,"side":"start","danger":true,"failed":true,"ready":false,"key":""},
  overflow: {"kind":"fever","value":1.4,"side":"end","danger":false,"failed":false,"ready":true,"key":"Left Shoulder + Right Shoulder"},
};
