import type { Texture } from "@origingame/engine";
import { Show, Text, type UiEvent, View, packClass, t } from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Backdrop } from "../controls/backdrop";
import { InputGlyph } from "../controls/input-glyph";
import "./title.en.strings.ts";

export interface TitleScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The build's version. */
  version: string;
  /** The legal line at the foot (copyright, trademarks). */
  legal: string;
  /** The key art's texture under the title (og.load.texture; 0: the style's dark ground). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. */
  keyArt: Texture | 0;
  /** The game's logo texture (og.load.texture; 0: the name set as a wordmark). */
  logo: Texture | 0;
  /** The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). */
  name: string;
  /** Content the caller supplies (was the background snippet). */
  background?: (() => JSX.Element) | undefined;
  /** The start event, with the row's key where it comes from a row. */
  onStart?: ((e: UiEvent, key?: string) => void) | undefined;
}

/**
 * The title screen over the key art, staged like a console attract screen:
 * it fades up from black while the art pushes in slowly, the logo (or the
 * name as a wide-tracked wordmark) rises into the middle, then Press any
 * button (Tap to start on a touch screen) arrives at the foot and pulses,
 * the confirm binding's glyph beside it. The legal line and the version sit in the lower corners. Any
 * press reports onStart. It resolves in the dark scheme in both schemes.
 */
export function TitleScreen(props: TitleScreenProps) {
  return (
    <View class={`absolute inset-0 scheme-dark text-fg p-0 overflow-hidden ${packClass("Screen", "frame")} ${props.class ?? ""}`} autofocus onPress={(e) => props.onStart?.(e)} onAction={{ confirm: (e) => props.onStart?.(e) }}>
      <View class="absolute inset-0 transition-transform duration-orbit ease-out starting:scale-lift">
        <Backdrop art={props.keyArt} />
        <View class="absolute inset-0">{props.background?.()}</View>
      </View>
      <View class="absolute inset-0 bg-linear-to-b from-material-thin via-transparent via-30% to-transparent" />
      <View class="absolute inset-0 bg-linear-to-t from-bg via-scrim via-25% to-transparent to-60%" />
      <View class="grid grid-cols-1 grid-rows-[1fr_auto_1fr_auto_auto] justify-items-center h-full pt-12 pb-8 px-14 portrait:px-8 compact:pt-8 compact:pb-5 compact:px-5">
        <View class="row-[2] items-center max-w-full transition duration-pulse delay-spring ease-out starting:opacity-0 starting:translate-y-6">
          <View class="absolute -inset-x-24 -inset-y-16 bg-radial from-bg via-scrim via-30% to-transparent to-70% pointer-events-none compact:-inset-x-8" />
          <Show when={props.logo === 0 && props.name !== ""} fallback={
              <Art class={`aspect-12/5 h-auto object-contain compact:w-43/50`} kind="logo" src={props.logo} />
          }>
            <Text class={`max-w-full text-center m-0 ${packClass("Title", "wordmark")} compact:text-display-xl`}>
              {props.name}
            </Text>
          </Show>
        </View>
        <Show when={props.onStart}>
          <View class="row-[4] mb-10 transition duration-bounce delay-breathe ease-out starting:opacity-0 starting:translate-y-3 compact:mb-8">
            <View class={`animate-pulse ${packClass("Screen", "prompt")}`}>
              <InputGlyph class="pointer-coarse:hidden" action="confirm" />
              <Text class="m-0 pointer-coarse:hidden">
                {t("screen.title.press")}
              </Text>
              <Text class="hidden m-0 pointer-coarse:flex">
                {t("screen.title.tap")}
              </Text>
            </View>
          </View>
        </Show>
        <View class="row-[5] flex-row justify-between items-end self-stretch gap-x-6 transition-opacity duration-bounce delay-breathe ease-out starting:opacity-0 compact:flex-col compact:items-center compact:gap-y-1">
          <Text class={`m-0 compact:text-center ${packClass("Screen", "note")}`}>{props.legal}</Text>
          <Text class={`m-0 tabular-nums ${packClass("Screen", "note")}`}>{props.version}</Text>
        </View>
      </View>
      <View class="absolute inset-0 pointer-events-none bg-bg opacity-0 transition-opacity duration-ping ease-out starting:opacity-100" />
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<TitleScreenProps>> = {
  default: {"keyArt":0,"legal":"© Studio. All rights reserved.","logo":0,"name":"Lantern Coast","version":"1.0.0","onStart":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Start"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"title","layer":"menu","focus":"","under":"none","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
