import type { Texture } from "@origingame/engine";
import {Scroll, For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./mode-select.en.ogt";

export interface ModeSelectScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The modes: { name, line, tag, locked, reason, texture }. */
  modes: ReadonlyArray<{ id: string; line: string; locked: boolean; name: string; reason: string; tag: string; texture: Texture | 0 }>;
  /** The stable collection key selected now. */
  selected: string;
  /** The key art's texture under the screen's veil (og.load.texture; 0: the style's dark ground). */
  keyArt: Texture | 0;
  /** Content the caller supplies (was the detail snippet). */
  detail?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The play event, with the row's key where it comes from a row. */
  onPlay?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

/**
 * A mode select: the modes as cards with their line, tag and picture, a
 * locked mode's reason, the selected one's detail, and Play. It reports
 * onSelect and onPlay; the game decides what is locked.
 */
export function ModeSelectScreen(props: ModeSelectScreenProps) {
  return (
    <View class={`absolute inset-0 pt-8 pb-6 text-fg gap-y-6 px-12 overflow-hidden compact:pt-4 compact:pb-3 compact:gap-y-3 compact:px-4 ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <Show when={props.keyArt !== 0}>
        <Art class="absolute object-cover inset-0 size-full" kind="keyArt" src={props.keyArt} />
      </Show>
      <View class={`absolute bg-glass inset-0`} />
      <View class={`flex-row min-h-12 border-b border-rim-soft items-center gap-x-4 py-1 px-4 narrow:flex-wrap narrow:gap-y-1`}>
        <Show when={props.onBack}>
          <Button class={`narrow:-ml-2 shrink`} onPress={(e) => props.onBack?.(e)}>
            <View class="flex-row items-center gap-x-1">
              <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
              <Text class="shrink">{t("screen.common.back")}</Text>
            </View>
          </Button>
        </Show>
        <Text class={`font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg shrink m-0 narrow:text-3xl narrow:grow narrow:shrink-0 narrow:basis-full`}>
          {t("screen.mode-select.title")}
        </Text>
        <View class="grow shrink basis-0 h-px bg-linear-to-r from-sheen to-transparent to-80% m-0 compact:hidden" />
      </View>
      <Scroll class="grow shrink basis-0 p-4 -m-4 narrow:pb-6">
        <View class="grid grid-cols-3 auto-rows-[1fr] h-full gap-6 narrow:grid-cols-1 narrow:auto-rows-auto narrow:h-auto narrow:gap-3" focus-group="wrap">
          <For each={props.modes}>
            {(mode) => (
            <View class={`group/mode min-h-80 narrow:min-h-42.5 text-fg transition duration-317 data-selected:border-accent data-done:border-success active:border-accent disabled:text-fg-subtle disabled:scale-100 active:scale-96 active:translate-x-0 active:translate-y-0 data-locked:scale-100 data-down:opacity-45 hover:translate-x-0 hover:-translate-y-0.25`} onPress={(e) => props.onSelect?.(e, mode().id)} tabIndex={0} data-selection-off={mode().id !== props.selected} data-selection-on={mode().id === props.selected} data-lock-open={!mode().locked} data-lock-locked={mode().locked}>
              <View class={`grow shrink justify-end nine-panel-cut overflow-hidden`}>
                <Art class="absolute object-cover inset-0 size-full" kind="levelThumb" src={mode().texture} />
                <View class={`absolute bg-linear-to-b from-transparent from-35% via-scrim via-60% to-shade-deep inset-0`} />
                <Show when={mode().tag}>
                  <Text class={`text-shadow-tag bg-tile nine-chip-cut absolute top-4 right-4 text-xs font-bold tracking-label uppercase text-fg whitespace-nowrap text-ellipsis py-0.5 px-2 overflow-hidden data-fresh:bg-hud-accent data-fresh:bg-none data-fresh:nine-chip-cut data-fresh:text-hud-accent-fg compact:top-2 compact:right-2 ${packClass("Chip", "frame")}`} data-fresh={mode().locked}>
                    {mode().tag}
                  </Text>
                </Show>
                <View class="gap-y-1 p-6 compact:gap-y-0.5 compact:p-3">
                  <Text class={`tracking-xs text-shadow-xs text-shadow-bg text-6xl leading-tight font-bold m-0 compact:text-display-sm`}>
                    {mode().name}
                  </Text>
                  <Text class="text-base leading-snug text-fg-muted line-clamp-2 m-0 compact:text-base">
                    {mode().line}
                  </Text>
                  <Show when={mode().locked}>
                    <View class={`flex-row items-center gap-x-1 text-base font-semibold text-warn`}>
                      <Icon class="size-4" name="lock" />
                      <Text class="shrink">{mode().reason}</Text>
                    </View>
                  </Show>
                </View>
                <View class="absolute inset-shadow-lit inset-ring-1 inset-ring-rim inset-0 group-data-selection-on/mode:inset-shadow-lit group-data-selection-on/mode:inset-ring-2 group-data-selection-on/mode:inset-ring-hud-accent" />
              </View>
            </View>
            )}
          </For>
        </View>
      </Scroll>
      <View>{props.detail?.()}</View>
      <View class="flex-row justify-end">
        <Show when={props.onPlay}>
          <Button variant="primary" class={`min-w-60 h-9 compact:min-w-0 compact:grow compact:shrink compact:basis-0`} disabled={props.modes.length === 0} onPress={(e) => props.onPlay?.(e)} onAction={{ confirm: (e) => props.onPlay?.(e) }}>
            <Text>{t("screen.mode-select.play")}</Text>
          </Button>
        </Show>
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<ModeSelectScreenProps>> = {
  default: {"keyArt":0,"modes":[{"id":"item-0","line":"The campaign, alone or with a friend.","locked":false,"name":"Story","reason":"","tag":"","texture":0},{"id":"item-1","line":"Ranked fights, one life.","locked":false,"name":"Arena","reason":"","tag":"Ranked","texture":0},{"id":"item-2","line":"Hard challenges for the brave.","locked":true,"name":"Trials","reason":"Finish chapter 2","tag":"New","texture":0}],"selected":"item-0","onBack":press,"onPlay":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Play","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"mode-select","layer":"menu","focus":"Modes","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
