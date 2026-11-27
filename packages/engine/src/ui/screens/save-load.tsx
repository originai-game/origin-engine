import type { Texture } from "@origingame/engine";
import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./save-load.en.strings.ts";

export interface SaveLoadScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** save or load: the title and the main action. */
  mode: string;
  /** The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }. */
  slots: ReadonlyArray<{ auto: boolean; date: string; empty: boolean; id: string; place: string; played: number; thumb: Texture | 0; title: string }>;
  /** The stable collection key selected now. */
  selected: string;
  /** Content the caller supplies (was the preview snippet). */
  preview?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The delete event, with the row's key where it comes from a row. */
  onDelete?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The load event, with the row's key where it comes from a row. */
  onLoad?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The newSlot event, with the row's key where it comes from a row. */
  onNewSlot?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The save event, with the row's key where it comes from a row. */
  onSave?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

/**
 * Saving or loading: the slots with their title, place, time played, date
 * and thumbnail (autosaves marked, empty slots offered for a new save), the
 * selected one's preview, and Save or Load and Delete. The game writes the
 * saves.
 */
export function SaveLoadScreen(props: SaveLoadScreenProps) {
  const selectedFull = () => props.slots.some((s) => s.id === props.selected && !s.empty);
  return (
    <View class={`absolute inset-0 pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+1.5rem)] text-fg gap-y-6 pl-[calc(env(hud-inset-left)+3rem)] pr-[calc(env(hud-inset-right)+3rem)] overflow-hidden compact:pt-[calc(env(hud-inset-top)+1rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:gap-y-3 compact:pl-[calc(env(hud-inset-left)+1rem)] compact:pr-[calc(env(hud-inset-right)+1rem)] ${packClass("Screen", "frame")} ${props.class ?? ""}`} onAction={{ back: (e) => props.onBack?.(e) }}>
      <View class={`absolute bg-glass backdrop-blur-regular inset-0`} />
      <View class={`flex-row min-h-12 items-center gap-x-4 py-1 px-4 narrow:flex-wrap narrow:gap-y-1`}>
        <Show when={props.onBack}>
          <Button class={`text-fg-muted narrow:-ml-2 shrink`} autofocus onPress={(e) => props.onBack?.(e)}>
            <View class="flex-row items-center gap-x-1">
              <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
              <Text class="shrink">{t("screen.common.back")}</Text>
            </View>
          </Button>
        </Show>
        <Show when={props.mode === "save"} fallback={
            <Text class={`font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg shrink m-0 compact:text-3xl narrow:grow narrow:shrink-0 narrow:basis-full`}>
              {t("screen.save-load.loadTitle")}
            </Text>
        }>
          <Text class={`font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg shrink m-0 compact:text-3xl narrow:grow narrow:shrink-0 narrow:basis-full`}>
            {t("screen.save-load.saveTitle")}
          </Text>
        </Show>
      </View>
      <View class="grow shrink basis-0 grid grid-cols-[minmax(0,3fr)_minmax(0,2fr)] grid-rows-1 gap-x-12 narrow:grid-cols-1 narrow:grid-rows-[minmax(0,1fr)_auto] narrow:gap-y-3">
        <Scroll class="p-1 -m-1">
          <View class={`gap-y-2 ${packClass("List", "cascade")}`} focusGroup="stop">
            <For each={props.slots}>
              {(s) => (
              <View class={`flex-row group/slot text-fg transition duration-control items-center gap-x-4 p-2 hover:bg-surface-hover data-empty-slot:justify-center data-empty-slot:min-h-26 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-dim ${s().id === props.selected ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} ${packClass("List", "item")}`} onPress={(e) => props.onSelect?.(e, s().id)} tabIndex={0} data-selection-off={s().id !== props.selected} data-selection-on={s().id === props.selected} data-empty-slot={s().empty}>
                <Art class={`w-37.5 aspect-89/50 h-auto object-cover compact:w-24 group-data-empty-slot/slot:hidden`} kind="saveThumb" src={s().thumb} />
                <View class="grow shrink basis-0 gap-y-0.5 pr-12 group-data-empty-slot/slot:pr-0 compact:pr-0">
                  <Show when={s().empty} fallback={
                    <>
                      <Text class={`text-xl leading-tight font-bold line-clamp-1 text-ellipsis m-0 compact:text-base`}>
                        {s().title}
                      </Text>
                      <Text class="line-clamp-1 text-ellipsis text-base text-fg-muted m-0">{s().place}</Text>
                      <View class="flex-row flex-wrap gap-x-3">
                        <Text class="text-caption tabular-nums text-fg-muted m-0 compact:whitespace-nowrap shrink">
                          {t("screen.save-load.Played", { s_played: formatDuration(s().played) })}
                        </Text>
                        <Text class="text-caption tabular-nums text-fg-muted m-0 compact:whitespace-nowrap shrink">
                          {s().date}
                        </Text>
                      </View>
                    </>
                  }>
                    <Text class="text-base text-fg-muted text-center m-0">
                      {t("screen.save-load.empty")}
                    </Text>
                  </Show>
                </View>
                <Show when={s().auto}>
                  <Text class={`text-shadow-tag bg-tile absolute top-2 right-3 text-xs font-bold tracking-label uppercase inset-shadow-lit py-0.5 px-2 rounded-control compact:right-auto compact:left-2`}>
                    {t("screen.save-load.auto")}
                  </Text>
                </Show>
              </View>
              )}
            </For>
          </View>
        </Scroll>
        <Scroll class={`gap-y-2 p-4 narrow:bg-none narrow:bg-transparent narrow:shadow-none narrow:inset-ring-0 narrow:p-0 data-blank:bg-none data-blank:bg-transparent data-blank:shadow-none data-blank:inset-ring-0 ${packClass("Panel", "frame")}`} data-blank={props.selected === ""}>
          <For each={props.slots}>
            {(s) => (
            <Show when={s().id === props.selected}>
              <Art class={`w-full aspect-89/50 h-auto object-cover mb-1 rounded-card inset-ring-hairline inset-ring-border narrow:hidden`} kind="saveThumb" src={s().thumb} />
              <Show when={s().empty} fallback={
                <>
                  <Text class="text-title leading-tight font-bold line-clamp-2 m-0 narrow:hidden">
                    {s().title}
                  </Text>
                  <View class="flex-row flex-wrap gap-x-3 gap-y-0.5 narrow:hidden">
                    <Text class="text-base tabular-nums text-fg-muted grow shrink basis-full line-clamp-2 m-0">
                      {s().place}
                    </Text>
                    <Text class="text-base tabular-nums text-fg-muted m-0 shrink">
                      {t("screen.save-load.Played", { s_played: formatDuration(s().played) })}
                    </Text>
                    <Text class="text-base tabular-nums text-fg-muted m-0 shrink">{s().date}</Text>
                  </View>
                </>
              }>
                <Text class="text-lg text-fg-muted m-0 narrow:hidden">
                  {t("screen.save-load.empty")}
                </Text>
              </Show>
            </Show>
            )}
          </For>
          <View class={`shrink narrow:hidden`}>{props.preview?.()}</View>
          <View class={`flex-row flex-wrap justify-end mt-auto pt-3 gap-2 narrow:pt-0`}>
            <Show when={props.onDelete}>
              <Button variant="danger" class="grow shrink basis-30 narrow:basis-25" disabled={!selectedFull()} onPress={(e) => { if (selectedFull()) props.onDelete?.(e); }} onAction={{ details: (e) => { if (selectedFull()) props.onDelete?.(e); } }}>
                <Text>{t("screen.save-load.delete")}</Text>
              </Button>
            </Show>
            <Show when={props.mode === "save"} fallback={
                <Show when={props.slots.length > 0}>
                  <Show when={props.onLoad}>
                    <Button variant="primary" class="grow shrink basis-30 narrow:basis-25" disabled={!selectedFull()} onPress={(e) => { if (selectedFull()) props.onLoad?.(e); }}>
                      <Text>{t("screen.save-load.load")}</Text>
                    </Button>
                  </Show>
                </Show>
            }>
              <Show when={props.onNewSlot}>
                <Button class="grow shrink basis-30 narrow:basis-25" onPress={(e) => props.onNewSlot?.(e)}>
                  <Text>{t("screen.save-load.new")}</Text>
                </Button>
              </Show>
              <Show when={props.onSave}>
                <Button variant="primary" class="grow shrink basis-30 narrow:basis-25" onPress={(e) => props.onSave?.(e)}>
                  <Text>{t("screen.save-load.save")}</Text>
                </Button>
              </Show>
            </Show>
          </View>
        </Scroll>
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<SaveLoadScreenProps>> = {
  default: {"mode":"load","selected":"item-0","slots":[{"auto":true,"date":"Today 18:42","empty":false,"id":"item-0","place":"Old mill","played":15120,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"item-1","place":"Harbour","played":9800,"thumb":0,"title":"Chapter 2"},{"auto":false,"date":"","empty":true,"id":"item-2","place":"","played":0,"thumb":0,"title":""}],"onBack":press,"onDelete":press,"onLoad":press,"onNewSlot":press,"onSave":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Back","details":"Delete"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"save-load","layer":"menu","focus":"Body/Slots/List","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
