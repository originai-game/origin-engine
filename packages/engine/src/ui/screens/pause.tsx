import {Scroll, Icon, Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import "./pause.en.ogt";

export interface PauseScreenProps {
  /** The current objective (empty: none). */
  objective: string;
  /** Seconds played this run. */
  played: number;
  /** Saving is allowed here. */
  canSave?: boolean;
  /** Content the caller supplies (was the side snippet). */
  side?: (() => JSX.Element) | undefined;
  /** The controls event, with the row's key where it comes from a row. */
  onControls?: (e: UiEvent, key?: string) => void;
  /** The load event, with the row's key where it comes from a row. */
  onLoad?: (e: UiEvent, key?: string) => void;
  /** The quit event, with the row's key where it comes from a row. */
  onQuit?: (e: UiEvent, key?: string) => void;
  /** The quitToMenu event, with the row's key where it comes from a row. */
  onQuitToMenu?: (e: UiEvent, key?: string) => void;
  /** The resume event, with the row's key where it comes from a row. */
  onResume?: (e: UiEvent, key?: string) => void;
  /** The save event, with the row's key where it comes from a row. */
  onSave?: (e: UiEvent, key?: string) => void;
  /** The settings event, with the row's key where it comes from a row. */
  onSettings?: (e: UiEvent, key?: string) => void;
}

export function PauseScreen(props: PauseScreenProps) {
  return (
    <View class="flex-row text-fg overflow-hidden">
      <View class={`absolute inset-0 compact:bg-linear-to-b compact:from-black/70 compact:via-black/86 compact:via-40% compact:to-black/96 compact:bg-transparent`} />
      <Scroll class={`pr-8 pl-12 text-fg w-160 max-w-full h-full transition duration-240 ease-out py-8 compact:pt-8 compact:pb-4 compact:w-full compact:px-4 short:py-3 short:px-4 disabled:text-fg-subtle starting:opacity-0 starting:-translate-x-6 starting:translate-y-0 shrink`}>
        <Text class={`font-display text-8xl font-bold leading-tight tracking-xs uppercase text-fg text-shadow-xs text-shadow-bg/60 m-0 compact:text-[2.857rem]`}>
          {t("screen.pause.title")}
        </Text>
        <View class={`gap-y-0.5 self-start w-[27.143rem] max-w-full mt-4 bg-plate-hi border border-white/18 nine-panel-cut py-3 px-4 short:hidden compact:self-stretch compact:w-auto compact:mt-3 compact:py-2 compact:px-3 ${packClass("Panel", "frame")}`}>
          <Show when={props.objective !== ""}>
            <Text class="text-[0.786rem] font-bold tracking-[0.24em] uppercase text-fg/56 m-0">
              {t("screen.pause.objectiveLabel")}
            </Text>
            <View class="flex-row items-center gap-x-2">
              <Text class="rotate-45 bg-hud-accent shadow-[0px_0px_11.424px_#38e8ff] size-[0.571rem]" />
              <Text class={`grow shrink text-[1.286rem] leading-snug font-semibold line-clamp-2 m-0 compact:text-[1.143rem]`}>
                {props.objective}
              </Text>
            </View>
          </Show>
          <Text class="mt-0.5 mb-0 text-base tabular-nums text-fg/74 mx-0">
            {t("screen.pause.Played", { played: formatDuration(props.played) })}
          </Text>
        </View>
        <View class="gap-y-0.5 mt-auto -ml-4 pt-6 short:grid short:pt-2 short:grid-cols-[1fr_1fr] short:gap-x-3 compact:-mx-4" focus-group="wrap">
          <View class={`flex-row items-center justify-center gap-1 group/resume w-full min-h-11 text-fg/74 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-fg active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent active:text-fg disabled:bg-transparent disabled:bg-none disabled:text-fg/32 compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onResume?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.resume")}
              </Text>
              <Icon class="text-hud-accent opacity-0 size-4 group-hover/resume:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </View>
          </View>
          <View class={`flex-row items-center justify-center gap-1 group/settings w-full min-h-11 text-fg/74 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-fg active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent active:text-fg disabled:bg-transparent disabled:bg-none disabled:text-fg/32 compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onSettings?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.settings")}
              </Text>
              <Icon class="text-hud-accent opacity-0 size-4 group-hover/settings:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </View>
          </View>
          <View class={`flex-row items-center justify-center gap-1 group/controls w-full min-h-11 text-fg/74 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-fg active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent active:text-fg disabled:bg-transparent disabled:bg-none disabled:text-fg/32 compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onControls?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.controls")}
              </Text>
              <Icon class="text-hud-accent opacity-0 size-4 group-hover/controls:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </View>
          </View>
          <View class={`flex-row items-center justify-center gap-1 group/save w-full min-h-11 text-fg/74 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-fg active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent active:text-fg disabled:bg-transparent disabled:bg-none disabled:text-fg/32 compact:pr-4 compact:pl-6`} role="button" disabled={!props.canSave} onPress={(e) => props.onSave?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.save")}
              </Text>
              <Icon class="text-hud-accent opacity-0 size-4 group-hover/save:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </View>
          </View>
          <View class={`flex-row items-center justify-center gap-1 group/load w-full min-h-11 text-fg/74 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-fg active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent active:text-fg disabled:bg-transparent disabled:bg-none disabled:text-fg/32 compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onLoad?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.load")}
              </Text>
              <Icon class="text-hud-accent opacity-0 size-4 group-hover/load:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
            </View>
          </View>
          <View class="h-px mt-2 mr-0 mb-1 ml-4 bg-linear-to-r from-white/28 to-transparent to-80% compact:ml-6 short:hidden" />
          <View class={`flex-row items-center justify-center gap-1 w-full min-h-11 text-fg/56 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent disabled:bg-transparent disabled:bg-none compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onQuitToMenu?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.quitToMenu")}
              </Text>
            </View>
          </View>
          <View class={`flex-row items-center justify-center gap-1 w-full min-h-11 text-fg/56 py-1 px-4 hover:bg-transparent hover:bg-hud-accent/24 hover:text-danger active:bg-transparent active:bg-linear-to-r active:from-hud-accent/34 active:via-hud-accent/12 active:via-55% active:to-transparent disabled:bg-transparent disabled:bg-none compact:pr-4 compact:pl-6`} role="button" onPress={(e) => props.onQuit?.(e)}>
            <View class="flex-row items-center gap-x-3 w-full">
              <Text class="grow shrink text-start text-[1.714rem] leading-tight font-semibold text-shadow-xs text-shadow-bg/60 compact:text-[1.429rem]">
                {t("screen.pause.quit")}
              </Text>
            </View>
          </View>
        </View>
      </Scroll>
      <View class="grow shrink basis-0 items-center justify-center p-8 compact:hidden">{props.side?.()}</View>
    </View>
  );
}

export const fixtures: Record<string, Partial<PauseScreenProps>> = {
  default: {"canSave":true,"objective":"Reach the lighthouse","played":4520},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Resume"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"pause","layer":"menu","focus":"Panel/Menu/Resume","under":"blur","input":"menu","pause":true,"capture":true,"remember":true,"back":"resume"} as const;
