import type { Texture } from "@origingame/engine";
import {Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { Spinner } from "../components/base/spinner";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./loading.en.ogt";

export interface LoadingScreenProps {
  /** The key art's texture (og.load.texture; 0: the style's placeholder). */
  keyArt: Texture | 0;
  /** Where the player is going. */
  place: string;
  /** A tip or a line of lore. */
  tip: string;
  /** 0 to 1. */
  progress: number;
  /** Loading is done. */
  ready?: boolean;
  /** The game waits for a press once ready. */
  waits?: boolean;
  /** What the wait is for, beside the figure while it is not ready (Waiting for the server); empty: none. */
  status?: string;
  /** Content the caller supplies (was the lore snippet). */
  lore?: (() => JSX.Element) | undefined;
  /** The continue event, with the row's key where it comes from a row. */
  onContinue?: (e: UiEvent, key?: string) => void;
  /** The nextTip event, with the row's key where it comes from a row. */
  onNextTip?: (e: UiEvent, key?: string) => void;
}

/**
 * The key art fills the screen. At its foot, over a shade that grows with
 * the words: where the player is going (an eyebrow and the place), the tip
 * on a cut smoked-glass plate, and the progress as the HUD's leaning bar
 * with a spinner and the figure; once ready, Continue (or nothing, when the
 * game does not wait). A press anywhere asks for the next tip.
 */
export function LoadingScreen(props: LoadingScreenProps) {
  return (
    <View class="text-fg overflow-hidden" onPress={(e) => props.onNextTip?.(e)}>
      <Show when={props.keyArt}>
        <Art class="absolute object-cover inset-0" kind="tutorial" src={props.keyArt} />
      </Show>
      <Show when={!props.keyArt}>
        <View class="absolute inset-0" />
      </Show>
      <View class={`absolute bg-linear-to-r from-black/45 to-transparent to-55% inset-0`} />
      <View class="absolute top-auto bottom-0 pt-16 inset-x-0 gap-[1.429rem] compact:pt-[1.429rem] compact:gap-4 short:pt-[1.143rem] short:gap-[0.714rem]">
        <View class="absolute -z-1 top-auto bottom-0 bg-linear-to-b from-transparent to-black/92 inset-x-0" />
        <View class="gap-[1.429rem] short:flex-row short:items-end short:gap-[1.143rem] compact:gap-4">
          <View class="max-w-3/5 gap-0.5 compact:max-w-full short:max-w-[45%] short:grow short:shrink short:basis-0 short:min-w-0">
            <View class={`items-center text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-hud-accent gap-[0.714rem] transition duration-240 ease-out starting:opacity-0 starting:translate-y-2`}>
              <View class="w-8 h-[0.143rem] bg-hud-accent shadow-[0px_0px_6.864px_#38e8ff]" />
              <Text class="shrink">{t("screen.loading.entering")}</Text>
            </View>
            <Text class={`font-display font-bold leading-[1.04] tracking-[-0.01em] text-fg text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg/55 m-0 transition duration-240 ease-out starting:opacity-0 starting:translate-y-2`}>
              {props.place}
            </Text>
          </View>
          <View class={`pt-[1.143rem] pb-[1.286rem] nine-panel-cut bg-plate-hi text-fg max-w-[54.286rem] px-[1.571rem] gap-1 compact:py-[0.857rem] compact:px-4 grow short:shrink short:basis-0 short:min-w-0 short:py-[0.714rem] short:px-4 disabled:text-fg-subtle transition duration-240 ease-out starting:opacity-0 starting:translate-y-2 ${packClass("Panel", "frame")}`}>
            <View class="flex-row items-center gap-2">
              <Text class={`tracking-[0.14em] uppercase text-hud-accent text-[0.857rem] font-bold bg-tile nine-plate-cut py-[0.143rem] px-[0.571rem] m-0 ${packClass("Chip", "frame")}`}>
                {t("screen.loading.tip")}
              </Text>
              <View class={`flex-row items-center justify-center shrink ml-auto gap-x-1 text-[0.857rem] font-semibold tracking-[0.14em] px-1 pointer-coarse:min-h-11`} role="button" onPress={(e) => props.onNextTip?.(e)}>
                <Text class="min-w-[1.571rem] text-[0.857rem] leading-[1.6] font-bold text-center text-hud-accent-fg bg-linear-to-b from-white to-key-lo shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] py-0 px-1 rounded-[4.576px] pointer-coarse:hidden">
                  {t("screen.loading.nextKey")}
                </Text>
                <Text>{t("screen.loading.next")}</Text>
              </View>
            </View>
            <Text class="text-body tracking-[0em] text-fg m-0">{props.tip}</Text>
            <View class="text-caption text-fg/66 short:hidden">{props.lore?.()}</View>
          </View>
        </View>
        <View class="items-center gap-[1.286rem] compact:items-stretch compact:flex-col-reverse compact:gap-[0.857rem]">
          <View class={`group/bar grow shrink h-[0.857rem] bg-track shadow-sunk gap-2 overflow-hidden compact:grow-0 compact:shrink-0 compact:basis-auto compact:w-full`} role="progressbar" label={props.place} valueMin={0} valueMax={1} valueNow={props.progress} data-ready={props.ready === true}>
            <View class={`absolute top-0 left-0 bg-linear-to-r from-hud-accent/55 to-hud-accent h-full shadow-[0px_0px_16px_#38e8ff99,inset_-3.424px_0px_0px_#ffffff] duration-400 group-data-ready/bar:shadow-[0px_0px_16px_#38e8ff99]`} style={{ width: `${(100 * (props.progress <= 0 ? 0 : props.progress >= 1 ? 1 : props.progress))}%` }} />
          </View>
          <View class="flex-row items-center gap-y-2 gap-x-[0.857rem] compact:flex-wrap">
            <Show when={!props.ready}>
              <Spinner size={22} name={t("screen.loading.busy")} />
            </Show>
            <Show when={!props.ready && props.status}>
              <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 m-0 shrink`}>
                {props.status}
              </Text>
            </Show>
            <Show when={!(props.ready && props.waits)}>
              <Text class={`tabular-nums text-fg text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg/55 min-w-[5.143rem] text-[2.857rem] leading-flat text-end font-bold whitespace-nowrap m-0 data-ready:text-hud-accent compact:ml-auto compact:min-w-0 compact:text-[2rem] short:text-[1.714rem] shrink`} data-ready={props.ready === true}>
                {t("screen.loading.Percent", { progress: `${formatNumber((props.progress) * 100, { decimals: 0 })}%` })}
              </Text>
            </Show>
            <Show when={props.ready && props.waits}>
              <Button class={`text-accent gap-x-2 min-h-11 font-bold px-[1.571rem] compact:grow compact:shrink compact:basis-auto animate-pop ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onContinue?.(e)} onAction={{ confirm: (e) => props.onContinue?.(e) }} autofocus>
                <Text class="min-w-[1.571rem] text-[0.857rem] leading-[1.6] font-bold text-center text-fg bg-hud-accent-fg uppercase tracking-[0.14em] whitespace-normal py-0 px-1 rounded-[4.576px] pointer-coarse:hidden">
                  {t("screen.loading.continueKey")}
                </Text>
                <Text class="uppercase tracking-[0.14em] whitespace-normal pointer-coarse:hidden">
                  {t("screen.loading.continue")}
                </Text>
                <Text class="uppercase tracking-[0.14em] hidden whitespace-normal pointer-coarse:flex">
                  {t("screen.loading.tapContinue")}
                </Text>
              </Button>
            </Show>
          </View>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<LoadingScreenProps>> = {
  default: {"keyArt":0,"place":"The Sunken Archive","progress":0.62,"ready":false,"tip":"Dodging through an attack at the last moment slows time for a breath.","waits":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"loading","layer":"modal","focus":"","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
