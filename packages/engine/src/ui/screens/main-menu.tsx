import type { Texture } from "@origingame/engine";
import {Scroll, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { MenuEntry, MenuList } from "../controls/menu-list";
import "./main-menu.en.ogt";

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
      <View class="absolute inset-0 bg-linear-to-r from-bg via-material-thick via-26% to-transparent to-70% compact:hidden" />
      <View class="absolute inset-0 bg-linear-to-b from-transparent from-60% to-bg compact:hidden" />
      <View class="absolute inset-0 hidden compact:flex bg-linear-to-b from-material-regular via-material-regular via-48% to-bg to-70%" />
      <Scroll class={`pt-[calc(env(hud-inset-top)+3rem)] pr-[calc(env(hud-inset-right)+2rem)] pb-[calc(env(hud-inset-bottom)+2rem)] pl-[calc(env(hud-inset-left)+3rem)] text-fg size-full compact:pt-[calc(env(hud-inset-top)+2rem)] compact:pb-[calc(env(hud-inset-bottom)+1.5rem)] compact:w-auto compact:pl-[calc(env(hud-inset-left)+1rem)] compact:pr-[calc(env(hud-inset-right)+1rem)] short:pt-[calc(env(hud-inset-top)+0.75rem)] short:pb-[calc(env(hud-inset-bottom)+0.75rem)] short:pl-[calc(env(hud-inset-left)+1rem)] short:pr-[calc(env(hud-inset-right)+1rem)] disabled:text-fg-subtle`}>
        <Show when={props.logo === 0 && props.name !== ""} fallback={
            <Art class={`self-start aspect-12/5 h-auto object-contain compact:self-center compact:w-39/50 short:hidden`} kind="logo" src={props.logo} />
        }>
          <Text class={`tracking-sm uppercase text-shadow-xs text-shadow-bg self-start max-w-full text-display-xl leading-tight font-bold m-0 compact:self-center compact:text-5xl compact:text-center short:hidden`}>
            {props.name}
          </Text>
        </Show>
        <Show when={props.news !== ""}>
          <View class={`flex-row absolute right-12 bottom-12 items-center gap-x-3 w-90 py-3 px-4 compact:static compact:w-auto compact:max-w-none compact:mt-auto compact:mb-3 compact:py-2 compact:px-3 short:hidden hover:translate-x-0 hover:-translate-y-0.25 active:translate-x-0 active:translate-y-0 ${packClass("Panel", "frame")}`} onPress={(e) => props.onNews?.(e)} tabIndex={props.onNews ? 0 : -1}>
            <View class="gap-y-0.5 grow shrink">
              <Text class="text-xs font-bold tracking-kicker uppercase text-hud-accent-ink m-0">
                {t("screen.main-menu.newsEyebrow")}
              </Text>
              <Text class="text-xl leading-snug font-semibold line-clamp-2 m-0 compact:text-base">
                {props.news}
              </Text>
            </View>
            <Show when={props.onNews}>
              <Icon class="text-fg-muted size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </Show>
          </View>
        </Show>
        <MenuList class="w-110 max-w-full mt-auto -ml-4 short:gap-y-0 compact:w-auto compact:max-w-none compact:-mx-4 compact:data-after-news:mt-0" data-after-news={props.news !== ""}>
          <Show when={props.canContinue && props.onContinue}>
            <MenuEntry id="continue" class="compact:pl-6" label={t("screen.main-menu.continue")} line={props.saveLine} onPress={(e) => props.onContinue?.(e)} autofocus />
          </Show>
          <Show when={props.onNewGame}>
            <MenuEntry id="newGame" class="compact:pl-6" label={t("screen.main-menu.newGame")} onPress={(e) => props.onNewGame?.(e)} autofocus={!props.canContinue} />
          </Show>
          <Show when={props.onModes}>
            <MenuEntry id="modes" class="compact:pl-6" label={t("screen.main-menu.modes")} onPress={(e) => props.onModes?.(e)} />
          </Show>
          <Show when={props.onSettings}>
            <MenuEntry id="settings" class="compact:pl-6" label={t("screen.main-menu.settings")} onPress={(e) => props.onSettings?.(e)} />
          </Show>
          <Show when={(props.onNewGame || props.onModes || props.onSettings || (props.canContinue && props.onContinue)) && (props.onCredits || props.onQuit)}>
            <View class="h-4" />
          </Show>
          <Show when={props.onCredits}>
            <MenuEntry id="credits" class="compact:pl-6" label={t("screen.main-menu.credits")} onPress={(e) => props.onCredits?.(e)} minor />
          </Show>
          <Show when={props.onQuit}>
            <MenuEntry id="quit" class="compact:pl-6" label={t("screen.main-menu.quit")} onPress={(e) => props.onQuit?.(e)} minor />
          </Show>
          <View>{props.extra?.()}</View>
        </MenuList>
      </Scroll>
      <Text class={`absolute right-4 bottom-3 text-xs tabular-nums text-fg-muted m-0 compact:right-3 compact:bottom-1`}>
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
