import type { Texture } from "@origingame/engine";
import {Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
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
  /** The key art's texture behind the menu (og.load.texture; 0: the style's dark ground). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. */
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
      <Show when={props.keyArt !== 0}>
        <Art class="absolute object-cover inset-0 size-full" kind="keyArt" src={props.keyArt} />
      </Show>
      <View class="absolute inset-0">{props.background?.()}</View>
      <View class={`absolute inset-y-0 left-0 w-3/5 bg-linear-to-r from-bg via-scrim to-transparent portrait:inset-x-0 portrait:top-auto portrait:h-3/4 portrait:w-full portrait:bg-linear-to-t portrait:via-scrim compact:inset-x-0 compact:top-auto compact:h-3/4 compact:w-full compact:bg-linear-to-t compact:via-scrim`} />
      <View class={`absolute inset-0 flex-col gap-6 pt-[calc(env(hud-inset-top)+3.5rem)] pb-[calc(env(hud-inset-bottom)+3rem)] pl-[calc(env(hud-inset-left)+4rem)] pr-[calc(env(hud-inset-right)+4rem)] portrait:items-stretch portrait:pt-[calc(env(hud-inset-top)+6rem)] portrait:pl-[calc(env(hud-inset-left)+2rem)] portrait:pr-[calc(env(hud-inset-right)+2rem)] compact:items-stretch compact:gap-4 compact:pt-[calc(env(hud-inset-top)+4rem)] compact:pb-[calc(env(hud-inset-bottom)+1.5rem)] compact:pl-[calc(env(hud-inset-left)+1.25rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)] short:pt-[calc(env(hud-inset-top)+1rem)] short:pb-[calc(env(hud-inset-bottom)+1rem)]`}>
        <Show when={props.logo === 0 && props.name !== ""} fallback={
            <Art class="self-start w-120 max-w-full aspect-12/5 h-auto object-contain portrait:self-center portrait:w-4/5 compact:self-center compact:w-4/5 short:w-72" kind="logo" src={props.logo} />
        }>
          <Text class={`self-start max-w-140 ${packClass("Title", "wordmark")} portrait:self-center portrait:text-center portrait:max-w-full compact:self-center compact:text-center compact:max-w-full compact:text-display-xl short:text-display-lg`}>
            {props.name}
          </Text>
        </Show>
        <View class="grow min-h-4" />
        <MenuList label={t("screen.main-menu.title")} class={`w-110 max-w-full p-3 ${packClass("Panel", "frame")} portrait:w-full compact:w-full`}>
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
            <View class="h-3 short:h-1" />
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
        <View class={`flex-row absolute right-16 bottom-12 items-center gap-x-3 w-90 py-3 px-4 portrait:hidden compact:hidden ${packClass("Plate", "frame")}`} onPress={(e) => props.onNews?.(e)} tabIndex={props.onNews ? 0 : -1}>
          <View class="gap-y-0.5 grow shrink">
            <Text class="text-xs font-bold tracking-kicker uppercase text-fg-muted m-0">
              {t("screen.main-menu.newsEyebrow")}
            </Text>
            <Text class="text-lg leading-snug font-semibold line-clamp-2 m-0">
              {props.news}
            </Text>
          </View>
          <Show when={props.onNews}>
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
          </Show>
        </View>
      </Show>
      <Text class={`absolute right-4 bottom-3 tabular-nums m-0 ${packClass("Screen", "note")} compact:right-3 compact:bottom-1`}>
        {props.version}
      </Text>
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
