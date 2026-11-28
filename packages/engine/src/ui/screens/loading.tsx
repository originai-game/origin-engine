import type { Texture } from "@origingame/engine";
import {Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { Spinner } from "../components/base/spinner";
import { Backdrop } from "../controls/backdrop";
import { Button } from "../controls/button";
import "./loading.en.strings.ts";
import { InputGlyph } from "../controls/input-glyph";
import { Progress } from "../controls/progress";

export interface LoadingScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
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
  const percent = () => (props.progress <= 0 ? 0 : props.progress >= 1 ? 1 : props.progress) * 100;
  return (
    <View class={`absolute inset-0 text-fg overflow-hidden ${packClass("Screen", "frame")} ${props.class ?? ""}`} onPress={(e) => props.onNextTip?.(e)}>
      <Backdrop art={props.keyArt} veil="foot" />
      <View class="absolute inset-x-0 bottom-0 flex-col gap-6 pb-[calc(env(hud-inset-bottom)+2rem)] pl-[calc(env(hud-inset-left)+5rem)] pr-[calc(env(hud-inset-right)+5rem)] portrait:gap-5 portrait:pl-[calc(env(hud-inset-left)+2rem)] portrait:pr-[calc(env(hud-inset-right)+2rem)] compact:gap-4 compact:pb-[calc(env(hud-inset-bottom)+1.25rem)] compact:pl-[calc(env(hud-inset-left)+1.25rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)]">
        <View class="flex-row items-end gap-10 portrait:flex-col portrait:items-stretch portrait:gap-5 compact:flex-col compact:items-stretch compact:gap-4">
          <View class="flex-col gap-1 grow shrink basis-0 min-w-0 transition duration-base ease-out starting:opacity-0 starting:translate-y-2 portrait:basis-auto compact:basis-auto">
            <Text class={`m-0 ${packClass("Screen", "note")} text-base uppercase tracking-kicker font-bold`}>{t("screen.loading.entering")}</Text>
            <Text class={`m-0 ${packClass("Screen", "heading")} compact:text-display-md`}>{props.place}</Text>
          </View>
          <View class="flex-col gap-2 w-130 max-w-full shrink transition duration-base ease-out starting:opacity-0 starting:translate-y-2 portrait:w-full compact:w-full">
            <View class="flex-row items-center gap-3">
              <Text class={`m-0 grow ${packClass("Screen", "note")} text-base font-bold tracking-kicker uppercase`}>{t("screen.loading.tip")}</Text>
              <View class={`flex-row items-center gap-x-2 pointer-coarse:min-h-11 ${packClass("Screen", "note")} text-base font-semibold`} role="button" onPress={(e) => props.onNextTip?.(e)}>
                <InputGlyph class="pointer-coarse:hidden" keyLabel={t("screen.loading.nextKey")} size="md" />
                <Text>{t("screen.loading.next")}</Text>
              </View>
            </View>
            <Text class={`m-0 ${packClass("Screen", "note")} text-xl leading-snug font-semibold normal-case tracking-normal`}>{props.tip}</Text>
            <View class="text-base text-fg-muted short:hidden">{props.lore?.()}</View>
          </View>
        </View>
        <View class="flex-row items-center gap-5 compact:gap-3">
          <Show when={!props.ready}>
            <Spinner size={24} name={t("screen.loading.busy")} />
          </Show>
          <Progress class="grow shrink" value={percent()} label={props.place} />
          <Show when={!props.ready && props.status}>
            <Text class={`m-0 shrink ${packClass("Screen", "note")} text-base uppercase tracking-label font-semibold`}>{props.status}</Text>
          </Show>
          <Show when={!(props.ready && props.waits)}>
            <Text class={`m-0 min-w-20 text-end tabular-nums whitespace-nowrap ${packClass("Screen", "heading")} text-display-sm compact:text-title`}>
              {t("screen.loading.Percent", { progress: `${formatNumber(percent(), { decimals: 0 })}%` })}
            </Text>
          </Show>
          <Show when={props.ready && props.waits}>
            <Button variant="primary" class="gap-x-2 animate-pop" onPress={(e) => props.onContinue?.(e)} onAction={{ confirm: (e) => props.onContinue?.(e) }} autofocus>
              <InputGlyph class="pointer-coarse:hidden" action="confirm" size="md" />
              <Text class="pointer-coarse:hidden">{t("screen.loading.continue")}</Text>
              <Text class="hidden pointer-coarse:flex">{t("screen.loading.tapContinue")}</Text>
            </Button>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<LoadingScreenProps>> = {
  default: {"keyArt":0,"place":"The Sunken Archive","progress":0.62,"ready":false,"tip":"Dodging through an attack at the last moment slows time for a breath.","waits":true},
  ready: {"keyArt":0,"place":"The Sunken Archive","progress":1,"ready":true,"tip":"Dodging through an attack at the last moment slows time for a breath.","waits":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"loading","layer":"modal","focus":"","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
