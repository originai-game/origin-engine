import type { Texture } from "@origingame/engine";
import { For, Show, Text, type UiEvent, View, t } from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Counter } from "../controls/counter";
import "./splash.en.ogt";

export interface SplashScreenProps {
  /** The marks in order: { name, texture } (texture: an og.load.texture handle; 0 sets the name as a wordmark). */
  marks: ReadonlyArray<{ id: string; name: string; texture: Texture | 0 }>;
  /** The stable collection key selected now. */
  mark: string;
  /** Boot progress, 0 to 1 (below 0: no bar). */
  progress: number;
  /** What is loading, small beside the bar's figure (empty: nothing). */
  status: string;
  /** Content the caller supplies (was the custom snippet). */
  custom?: (() => JSX.Element) | undefined;
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
}

export function SplashScreen(props: SplashScreenProps) {
  return (
    <View class={`text-fg items-center gap-y-6 p-12 compact:pt-8 compact:pb-15 compact:px-6`} onPress={(e) => props.onSkip?.(e)} onAction={{ confirm: (e) => props.onSkip?.(e) }}>
      <View class="items-center gap-y-6 w-full mt-auto">
        <For each={props.marks}>
          {(m) => (
          <View class={`items-center gap-y-3 w-full transition-opacity duration-240 ease-out data-away:hidden data-away:transition-opacity data-away:duration-180 data-away:ease-in data-away:opacity-0 starting:opacity-0 data-hidden:hidden`} data-hidden={m().id !== props.mark} label={m().name} data-away={m().id !== props.mark}>
            <Show when={m().texture !== 0} fallback={
                <Text class="max-w-full leading-tight font-bold tracking-[0.24em] uppercase text-center text-shadow-xs text-shadow-bg/60 m-0">
                  {m().name}
                </Text>
            }>
              <Art class="aspect-[2/1] h-auto object-contain compact:w-4/5" kind="logo" src={m().texture} />
            </Show>
          </View>
          )}
        </For>
        <Show when={props.marks.length > 1}>
          <View class={`gap-x-1`}>
            <For each={props.marks}>
              {(m) => (
              <Text class={`bg-white/20 text-fg w-[1.286rem] h-[1.5px] transition-colors duration-317 rounded-capsule data-on:bg-none data-on:bg-white shrink`} data-on={m().id === props.mark} />
              )}
            </For>
          </View>
        </Show>
      </View>
      <View class="mb-auto">{props.custom?.()}</View>
      <Show when={props.progress >= 0}>
        <View class={`gap-y-2 w-full max-w-120`} role="progressbar" valueMin={0} valueMax={1} valueNow={props.progress > 1 ? 1 : props.progress} label={props.status}>
          <View class="items-baseline gap-x-3">
            <Text class="grow shrink text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/56 whitespace-nowrap m-0 overflow-hidden">
              {props.status}
            </Text>
            <Counter class="text-[0.929rem] font-bold text-fg/74" value={props.progress > 1 ? 1 : props.progress} format="percent" decimals={0} />
          </View>
          <View class={`bg-white/10 shadow-sunk h-[0.286rem] rounded-capsule overflow-hidden`}>
            <View class={`bg-linear-to-r from-[#38e8ff99] to-white h-full shadow-[0px_0px_13.712px_#38e8ff] duration-317`} style={{ width: `${(100 * (props.progress > 1 ? 1 : props.progress))}%` }} />
          </View>
        </View>
      </Show>
      <View class={`absolute right-6 bottom-4 items-center gap-x-2 text-[0.929rem] text-fg/74 compact:bottom-4 compact:justify-center compact:inset-x-4`}>
        <Text class="text-[0.857rem] leading-[1.6] font-bold text-[#0b0f18] bg-linear-to-b from-white to-[#c9d1e2] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] py-0 px-1 rounded-[4.576px] pointer-coarse:hidden shrink">
          {t("screen.splash.skipKey")}
        </Text>
        <Text class="pointer-coarse:hidden shrink">{t("screen.splash.skip")}</Text>
        <Text class="hidden pointer-coarse:flex shrink">{t("screen.splash.tap")}</Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<SplashScreenProps>> = {
  default: {"mark":"item-0","marks":[{"id":"item-0","name":"Studio","texture":0},{"id":"item-1","name":"Engine","texture":0}],"progress":0.4,"status":"Loading shaders"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Skip"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"splash","layer":"modal","focus":"","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
