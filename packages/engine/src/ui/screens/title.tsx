import type { Texture } from "@origingame/engine";
import { Show, Text, type UiEvent, View, t } from "@origingame/engine/ui";
import { Art } from "../controls/art";
import "./title.en.ogt";

export interface TitleScreenProps {
  /** The build's version. */
  version: string;
  /** The legal line at the foot (copyright, trademarks). */
  legal: string;
  /** The key art's texture under the title (og.load.texture; 0: the style's dark ground). */
  keyArt: Texture | 0;
  /** The game's logo texture (og.load.texture; 0: the name set as a wordmark). */
  logo: Texture | 0;
  /** The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). */
  name: string;
  /** Content the caller supplies (was the background snippet). */
  background?: (() => JSX.Element) | undefined;
  /** The start event, with the row's key where it comes from a row. */
  onStart?: (e: UiEvent, key?: string) => void;
}

export function TitleScreen(props: TitleScreenProps) {
  return (
    <View class={`text-accent-hover p-0 overflow-hidden`} onPress={(e) => props.onStart?.(e)} onAction={{ confirm: (e) => props.onStart?.(e) }}>
      <Show when={props.keyArt !== 0}>
        <Art class="absolute object-cover inset-0 size-full" kind="keyArt" src={props.keyArt} />
      </Show>
      <View class="absolute inset-0">{props.background?.()}</View>
      <View class={`absolute inset-0`} />
      <View class="grid grid-cols-[minmax(0,1fr)] grid-rows-[1fr_auto_1fr_auto_auto_auto] justify-items-center gap-y-3 h-full pt-12 pb-6 px-12 compact:gap-y-2 compact:pt-8 compact:pb-4 compact:px-4">
        <Show when={props.logo === 0 && props.name !== ""} fallback={
            <Art class={`row-[2] aspect-[12/5] h-auto object-contain compact:w-[86%]`} kind="logo" src={props.logo} />
        }>
          <Text class={`tracking-sm uppercase text-shadow-xs text-shadow-bg/60 row-[2] max-w-full leading-tight font-bold text-center m-0`}>
            {props.name}
          </Text>
        </Show>
        <View class="row-[4] items-center gap-x-4 compact:gap-x-2">
          <Text class={`basis-[8.571rem] h-px bg-linear-to-r from-transparent to-[#38e8ffe5] shadow-[0px_0px_11.424px_#38e8ff] compact:basis-[2.286rem]`} />
          <View class={`grow shrink basis-0 text-[1.286rem] font-semibold tracking-[0.28em] uppercase text-center compact:text-base compact:tracking-wide`}>
            <Text class="text-shadow-sm text-shadow-bg/80 m-0 pointer-coarse:hidden">{t("screen.title.press")}</Text>
            <Text class="hidden text-shadow-sm text-shadow-bg/80 m-0 pointer-coarse:flex">
              {t("screen.title.tap")}
            </Text>
          </View>
          <Text class={`basis-[8.571rem] h-px bg-linear-to-r from-transparent to-[#38e8ffe5] shadow-[0px_0px_11.424px_#38e8ff] -scale-x-100 scale-y-100 compact:basis-[2.286rem]`} />
        </View>
        <Text class="row-[5] text-[0.857rem] tracking-sm text-accent-hover/56 text-center m-0">{props.legal}</Text>
        <Text class="row-[6] justify-self-end text-[0.857rem] tabular-nums text-accent-hover/56 m-0 compact:justify-self-center">
          {props.version}
        </Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<TitleScreenProps>> = {
  default: {"keyArt":0,"legal":"© Studio. All rights reserved.","logo":0,"name":"Lantern Coast","version":"1.0.0"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Start"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"title","layer":"menu","focus":"","under":"none","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
