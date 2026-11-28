import type { Texture } from "@origingame/engine";
import {Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Backdrop } from "../controls/backdrop";
import { HintBar } from "../controls/hint-bar";
import { MenuEntry, MenuList } from "../controls/menu-list";
import "./main-menu.en.strings.ts";

export interface MainMenuScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** A save exists: Continue shows and takes the focus. */
  canContinue?: boolean;
  /** Under Continue: where the last save stands (chapter, time played). */
  saveLine: string;
  /** The build's version, shown small in the corner. */
  version: string;
  /** A line for the news card (empty: no card). */
  news: string;
  /** The key art's texture behind the menu (og.load.texture; 0: the pack's designed backdrop, Screen.backdrop). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. */
  keyArt: Texture | 0;
  /** The game's logo texture (og.load.texture; 0: the name set as a wordmark). */
  logo: Texture | 0;
  /** The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). */
  name: string;
  /** Content the caller supplies (was the background snippet). */
  background?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the extra snippet). */
  extra?: (() => JSX.Element) | undefined;
  /** The continue event, with the row's key where it comes from a row. */
  onContinue?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The credits event, with the row's key where it comes from a row. */
  onCredits?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The modes event, with the row's key where it comes from a row. */
  onModes?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The newGame event, with the row's key where it comes from a row. */
  onNewGame?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The news event, with the row's key where it comes from a row. */
  onNews?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The quit event, with the row's key where it comes from a row. */
  onQuit?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The settings event, with the row's key where it comes from a row. */
  onSettings?: ((e: UiEvent, key?: string) => void) | undefined;
}

/**
 * The main menu over the key art: the logo or the name as a wordmark,
 * Continue with the last save's line when a save exists, New game, Modes,
 * Settings, Credits and Quit, each drawn only when the game passes its
 * handler, a news card (pressable with onNews) and the version. It resolves in
 * the dark scheme in both schemes.
 */
export function MainMenuScreen(props: MainMenuScreenProps) {
  return (
    <View class={`absolute inset-0 scheme-dark text-fg overflow-hidden ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <Backdrop art={props.keyArt} />
      <View class="absolute inset-0">{props.background?.()}</View>
      <Backdrop art={0} over veil="side" />
      <View class={`absolute inset-0 flex-col justify-center gap-10 pt-[calc(env(hud-inset-top)+3rem)] pb-[calc(env(hud-inset-bottom)+5rem)] pl-[calc(env(hud-inset-left)+5rem)] pr-[calc(env(hud-inset-right)+4rem)] portrait:justify-end portrait:items-stretch portrait:gap-8 portrait:pt-[calc(env(hud-inset-top)+4rem)] portrait:pb-[calc(env(hud-inset-bottom)+5rem)] portrait:pl-[calc(env(hud-inset-left)+2rem)] portrait:pr-[calc(env(hud-inset-right)+2rem)] compact:justify-end compact:items-stretch compact:gap-5 compact:pt-[calc(env(hud-inset-top)+2rem)] compact:pb-[calc(env(hud-inset-bottom)+3.5rem)] compact:pl-[calc(env(hud-inset-left)+1.25rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)] short:gap-4 short:pt-[calc(env(hud-inset-top)+1rem)] short:pb-[calc(env(hud-inset-bottom)+2.5rem)]`}>
        <Show when={props.logo === 0 && props.name !== ""} fallback={
            <Art class="self-start w-120 max-w-full aspect-12/5 h-auto object-contain portrait:self-center portrait:w-4/5 compact:self-center compact:w-4/5 short:w-72" kind="logo" src={props.logo} />
        }>
          <Text class={`self-start max-w-130 ${packClass("Title", "wordmark")} portrait:self-center portrait:text-center portrait:max-w-full compact:self-center compact:text-center compact:max-w-full compact:text-display-xl short:text-display-lg`}>
            {props.name}
          </Text>
        </Show>
        <MenuList label={t("screen.main-menu.title")} class={`w-120 max-w-full ${packClass("MenuList", "frame")} portrait:w-full compact:w-full`}>
          <Show when={props.canContinue && props.onContinue}>
            <MenuEntry id="continue" label={t("screen.main-menu.continue")} line={props.saveLine} onPress={(e) => props.onContinue?.(e)} autofocus />
          </Show>
          <Show when={props.onNewGame}>
            <MenuEntry id="newGame" label={t("screen.main-menu.newGame")} onPress={(e) => props.onNewGame?.(e)} autofocus={!props.canContinue} />
          </Show>
          <Show when={props.onModes}>
            <MenuEntry id="modes" label={t("screen.main-menu.modes")} onPress={(e) => props.onModes?.(e)} />
          </Show>
          <Show when={props.onSettings}>
            <MenuEntry id="settings" label={t("screen.main-menu.settings")} onPress={(e) => props.onSettings?.(e)} />
          </Show>
          <Show when={(props.onNewGame || props.onModes || props.onSettings || (props.canContinue && props.onContinue)) && (props.onCredits || props.onQuit)}>
            <View class="h-4 short:h-1" />
          </Show>
          <Show when={props.onCredits}>
            <MenuEntry id="credits" label={t("screen.main-menu.credits")} onPress={(e) => props.onCredits?.(e)} minor />
          </Show>
          <Show when={props.onQuit}>
            <MenuEntry id="quit" label={t("screen.main-menu.quit")} onPress={(e) => props.onQuit?.(e)} minor danger />
          </Show>
          <View>{props.extra?.()}</View>
        </MenuList>
      </View>
      <Show when={props.news !== ""}>
        <View class="absolute bottom-0 right-0 pb-[calc(env(hud-inset-bottom)+5rem)] pr-[calc(env(hud-inset-right)+4rem)] portrait:hidden compact:hidden">
          <View class={`flex-row items-center gap-x-3 w-90 py-3 px-4 ${packClass("Plate", "frame")}`} onPress={(e) => props.onNews?.(e)} tabIndex={props.onNews ? 0 : -1}>
            <View class="gap-y-0.5 grow shrink">
              <Text class="text-sm font-bold tracking-kicker uppercase text-fg-muted m-0">
                {t("screen.main-menu.newsEyebrow")}
              </Text>
              <Text class="text-lg leading-snug font-semibold line-clamp-2 m-0">
                {props.news}
              </Text>
            </View>
            <Show when={props.onNews}>
              <Icon class="size-5 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </Show>
          </View>
        </View>
      </Show>
      <View class="absolute inset-x-0 bottom-0 flex-row items-center justify-between gap-x-6 pb-[calc(env(hud-inset-bottom)+1.5rem)] pl-[calc(env(hud-inset-left)+5rem)] pr-[calc(env(hud-inset-right)+4rem)] portrait:pl-[calc(env(hud-inset-left)+2rem)] portrait:pr-[calc(env(hud-inset-right)+2rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:pl-[calc(env(hud-inset-left)+1.25rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)]">
        <HintBar hints={[{ action: "confirm", label: t("screen.common.select") }]} />
        <Text class={`tabular-nums m-0 ${packClass("Screen", "note")}`}>
          {props.version}
        </Text>
      </View>
    </View>
  );
}


/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<MainMenuScreenProps>> = {
  default: {"canContinue":true,"keyArt":0,"logo":0,"name":"Lantern Coast","news":"Season 2 is live","saveLine":"Chapter 3 · 4h 12m","version":"1.0.0","onContinue":press,"onNewGame":press,"onModes":press,"onSettings":press,"onCredits":press,"onNews":press,"onQuit":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"main-menu","layer":"menu","focus":"MenuMenu/Entries/Continue","under":"none","input":"menu","pause":false,"capture":true,"remember":true,"back":"none"} as const;
