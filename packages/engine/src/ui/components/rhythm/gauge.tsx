import { For, Show, Text, View, t } from "@origingame/engine/ui";
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
  const full = props.value <= 0 ? 0 : props.value >= 1 ? 1 : props.value;
  return (
    <View
      class={`group/gauge flex-col items-center gap-1 h-full min-h-[8.571rem] compact:min-h-0 max-w-[6.857rem] w-full text-fg`}
      data-kind-fever={(props.kind ?? "life") === "fever"}
      data-side-end={(props.side ?? "start") === "end"}
      data-danger={props.danger}
      data-ready={props.ready}
    >
      <View
        class={`relative flex-1 min-h-0 w-[1.857rem] compact:w-[1.143rem] overflow-hidden -skew-y-[12deg] group-data-side-end/gauge:skew-y-[12deg] bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff40,0px_3px_8px_#00000073] data-danger:shadow-[0px_0px_0px_1px_#ff4d5e,0px_0px_12px_#ff4d5eb3] data-danger:animate-pulse data-ready:shadow-[0px_0px_14px_#38e8ff,inset_0px_0px_0px_1px_#ffffff40]`}
        role="meter"
        label={(props.kind ?? "life") === "fever" ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")}
        valueMin={0}
        valueMax={100}
        valueNow={props.value <= 0 ? 0 : props.value >= 1 ? 100 : props.value * 100}
        data-danger={props.danger}
        data-ready={props.ready}
      >
        <View
          class={`absolute bottom-0 left-0 w-full bg-[#55e39a] shadow-[0px_0px_6px_#55e39a66] group-data-kind-fever/gauge:bg-[#ff3d8a] group-data-kind-fever/gauge:shadow-[0px_0px_6px_#ff3d8a66] group-data-danger/gauge:bg-[#de2634]`}
          style={{ height: `${full * 100}%` }}
        />
        <For each={Array.from({ length: 9 }, (_, i) => i)}>
          {(i) => <View class={`absolute left-0 right-0 h-[3px] bg-black/85`} style={{ top: `${(i() + 1) * 10}%`, marginTop: "-1.5px" }} />}
        </For>
      </View>
      <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] text-stroke-3 text-stroke-bg group-data-kind-fever/gauge:text-[#ff3d8a]`}>
        {(props.kind ?? "life") === "fever" ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")}
      </Text>
      <Show when={props.failed} fallback={
        <Show when={props.danger}>
          <Text role="status" class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-danger text-stroke-3 text-stroke-bg`}>
            {t("rhythm.gauge.danger")}
          </Text>
        </Show>
      }>
        <Text role="status" class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-danger text-stroke-3 text-stroke-bg`}>
          {t("rhythm.gauge.failed")}
        </Text>
      </Show>
      <Show when={props.ready}>
        <Text role="status" class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-[#ff3d8a] text-stroke-3 text-stroke-bg`}>
          {t("rhythm.gauge.ready")}
        </Text>
      </Show>
      <Show when={props.key !== ""}>
        <Text
          class={`max-w-full px-1 text-center text-[0.857rem] font-bold bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] data-ready:bg-linear-to-b data-ready:from-[#dffbff] data-ready:to-[#38e8ff] data-ready:shadow-[0px_0px_14px_#38e8ff,inset_0px_-2.288px_0px_#0000002e]`}
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
