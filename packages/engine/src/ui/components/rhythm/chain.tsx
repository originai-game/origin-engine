import { For, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmChainProps {
  /** The score multiplier as the game words it (×4); empty hides the chain. */
  multiplier: string;
  /** How many of the steps to the next multiplier are lit. */
  step: number;
  /** The steps to the next multiplier; 0 hides the strip (at its top multiplier). Default 8. */
  steps?: number;
  /** Changes when the multiplier rises: the plate pops again. */
  hit: number;
}

/**
 * The score chain: a lit plate with the multiplier, and beside it the
 * steps to the next one as leaning pips. The game counts both; read only.
 */
export function RhythmChain(props: RhythmChainProps) {
  return (
    <View class={`flex-row items-center gap-1 max-w-full`} role="group" label={t("rhythm.chain.name", { multiplier: props.multiplier })}>
      <Show when={props.multiplier !== ""}>
        <Show when={`${props.multiplier}·${props.hit}`} keyed>
          {() => (
          <View class={`flex-row items-baseline gap-1 py-0.5 px-3 shrink bg-plate-hi nine-plate-cut animate-pop ${packClass("Plate", "frame")}`}>
            <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-fg/78`}>
              {t("rhythm.chain.label")}
            </Text>
            <Text class={`text-[1.714rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-[#38e8ff]`}>
              {props.multiplier}
            </Text>
          </View>
          )}
        </Show>
        <Show when={(props.steps ?? 8) > 0}>
          <View
            class={`relative shrink -skew-x-[22deg]`}
            style={{ width: `${(props.steps ?? 8) * 15 - 3}px`, height: "5px" }}
            role="meter"
            label={t("rhythm.chain.steps")}
            valueMin={0}
            valueMax={props.steps ?? 8}
            valueNow={props.step < 0 ? 0 : props.step > (props.steps ?? 8) ? (props.steps ?? 8) : props.step}
          >
            <View class={`absolute inset-0 flex-row gap-[3px]`}>
              <For each={Array.from({ length: props.steps ?? 8 }, (_, i) => i)}>
                {() => <View class={`w-[12px] shrink-0 bg-white/22`} />}
              </For>
            </View>
            <View
              class={`absolute inset-y-0 left-0 overflow-hidden`}
              style={{ width: `${(props.step <= 0 ? 0 : props.step >= (props.steps ?? 8) ? 1 : props.step / (props.steps ?? 8)) * ((props.steps ?? 8) * 15 - 3)}px` }}
            >
              <View class={`h-full flex-row gap-[3px]`} style={{ width: `${(props.steps ?? 8) * 15 - 3}px` }}>
                <For each={Array.from({ length: props.steps ?? 8 }, (_, i) => i)}>
                  {() => <View class={`w-[12px] shrink-0 bg-[#38e8ff] shadow-[0px_0px_6px_#38e8ff]`} />}
                </For>
              </View>
            </View>
          </View>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmChainProps>> = {
  default: {"multiplier":"×4","step":6,"steps":8,"hit":1},
  empty: {"multiplier":"×1","step":0,"steps":8,"hit":0},
  top: {"multiplier":"×8","step":0,"steps":0,"hit":3},
  overflow: {"multiplier":"×128.75","step":19,"steps":20,"hit":4},
};
