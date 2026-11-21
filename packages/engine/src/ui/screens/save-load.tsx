import type { Texture } from "@origingame/engine";
import { For, Icon, Scroll, Show, Text, type UiEvent, View, formatDuration, t } from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./save-load.en.ogt";

export interface SaveLoadScreenProps {
  /** save or load: the title and the main action. */
  mode: string;
  /** The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }. */
  slots: ReadonlyArray<{ auto: boolean; date: string; empty: boolean; id: string; place: string; played: number; thumb: Texture | 0; title: string }>;
  /** The stable collection key selected now. */
  selected: string;
  /** Content the caller supplies (was the preview snippet). */
  preview?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The delete event, with the row's key where it comes from a row. */
  onDelete?: (e: UiEvent, key?: string) => void;
  /** The load event, with the row's key where it comes from a row. */
  onLoad?: (e: UiEvent, key?: string) => void;
  /** The newSlot event, with the row's key where it comes from a row. */
  onNewSlot?: (e: UiEvent, key?: string) => void;
  /** The save event, with the row's key where it comes from a row. */
  onSave?: (e: UiEvent, key?: string) => void;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

export function SaveLoadScreen(props: SaveLoadScreenProps) {
  return (
    <View class={`pt-8 pb-6 text-accent-hover gap-y-6 px-12 overflow-hidden compact:pt-4 compact:pb-3 compact:gap-y-3 compact:px-4`}>
      <View class={`absolute bg-linear-to-b from-[#03060ee5] via-[#03060ed1] via-40% to-[#03060ef0] inset-0`} />
      <View class={`min-h-12 border-b border-rim-soft items-center gap-x-4 py-1 px-4 narrow:flex-wrap narrow:gap-y-1`}>
        <Button class="text-accent-hover/74 narrow:-ml-2 shrink" onPress={(e) => props.onBack?.(e)}>
          <View class="items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
        <Show when={props.mode === "save"} fallback={
            <Text class={`font-display text-[2.857rem] font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg/60 shrink m-0 compact:text-[1.857rem] narrow:grow narrow:shrink-0 narrow:basis-full`}>
              {t("screen.save-load.loadTitle")}
            </Text>
        }>
          <Text class={`font-display text-[2.857rem] font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg/60 shrink m-0 compact:text-[1.857rem] narrow:grow narrow:shrink-0 narrow:basis-full`}>
            {t("screen.save-load.saveTitle")}
          </Text>
        </Show>
        <View class="grow shrink basis-0 h-px bg-linear-to-r from-accent-hover/28 to-transparent to-80% m-0 narrow:hidden" />
      </View>
      <View class="grow shrink basis-0 grid grid-cols-[minmax(0,3fr)_minmax(0,2fr)] grid-rows-[minmax(0,1fr)] gap-x-12 narrow:grid-cols-[minmax(0,1fr)] narrow:grid-rows-[minmax(0,1fr)_auto] narrow:gap-y-3">
        <Scroll class="p-1 -m-1">
          <View class="gap-y-2" focus-group="stop">
            <For each={props.slots}>
              {(s) => (
              <View class={`group/slot rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none bg-linear-to-r from-[#151d2eb8] to-[#0b101c80] text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#ffffff12] items-center gap-x-4 p-2 hover:bg-transparent hover:bg-linear-to-r hover:from-[#38e8ff3d] hover:via-[#38e8ff14] hover:via-55% hover:to-[#38e8ff08] data-selection-on:bg-transparent data-selection-on:bg-linear-to-r data-selection-on:from-[#38e8ff3d] data-selection-on:via-[#38e8ff14] data-selection-on:via-55% data-selection-on:to-[#38e8ff08] data-selection-on:shadow-[inset_0px_0px_0px_2.288px_#38e8ff,inset_0px_3.424px_0px_#ffffff80] data-empty-slot:bg-transparent data-empty-slot:bg-none data-empty-slot:shadow-[inset_0px_0px_0px_1px_#ffffff1f] data-empty-slot:justify-center data-empty-slot:min-h-[7.429rem] data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} onPress={(e) => props.onSelect?.(e, s().id)} tabIndex={0} data-selection-off={s().id !== props.selected} data-selection-on={s().id === props.selected} data-empty-slot={s().empty}>
                <Art class={`w-[10.714rem] aspect-[89/50] h-auto object-cover compact:w-[6.857rem] group-data-empty-slot/slot:hidden`} kind="saveThumb" src={s().thumb} />
                <View class="grow shrink basis-0 gap-y-0.5 pr-12 group-data-empty-slot/slot:pr-0 compact:pr-0">
                  <Show when={s().empty} fallback={
                    <>
                      <Text class={`text-[1.429rem] leading-tight font-bold line-clamp-1 text-ellipsis m-0 compact:text-[1.214rem]`}>
                        {s().title}
                      </Text>
                      <Text class="line-clamp-1 text-ellipsis text-base text-accent-hover/74 m-0">{s().place}</Text>
                      <View class="flex-wrap gap-x-3">
                        <Text class="text-[0.929rem] tabular-nums text-accent-hover/56 m-0 compact:whitespace-nowrap shrink">
                          {t("screen.save-load.Played", { s_played: formatDuration(s().played) })}
                        </Text>
                        <Text class="text-[0.929rem] tabular-nums text-accent-hover/56 m-0 compact:whitespace-nowrap shrink">
                          {s().date}
                        </Text>
                      </View>
                    </>
                  }>
                    <Text class="text-[1.143rem] text-accent-hover/56 text-center m-0">
                      {t("screen.save-load.empty")}
                    </Text>
                  </Show>
                </View>
                <Show when={s().auto}>
                  <Text class={`text-shadow-2xs bg-[#060910e0] absolute top-2 right-3 text-[0.857rem] font-bold tracking-[0.14em] uppercase shadow-[inset_0px_1px_0px_#ffffff38] py-0.5 px-2 rounded-control compact:right-auto compact:left-2`}>
                    {t("screen.save-load.auto")}
                  </Text>
                </Show>
              </View>
              )}
            </For>
          </View>
        </Scroll>
        <Scroll class={`gap-y-2 bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none p-4 narrow:bg-none narrow:bg-transparent narrow:shadow-none narrow:p-0 data-blank:bg-none data-blank:bg-transparent data-blank:shadow-none`} data-blank={props.selected === ""}>
          <For each={props.slots}>
            {(s) => (
            <Show when={s().id === props.selected}>
              <Art class={`w-full aspect-[89/50] h-auto object-cover mb-1 shadow-[inset_0px_0px_0px_1px_#ffffff1f] narrow:hidden`} kind="saveThumb" src={s().thumb} />
              <Show when={s().empty} fallback={
                <>
                  <Text class="text-[1.714rem] leading-tight font-bold line-clamp-2 m-0 narrow:hidden">
                    {s().title}
                  </Text>
                  <View class="flex-wrap gap-x-3 gap-y-0.5 narrow:hidden">
                    <Text class="text-base tabular-nums text-accent-hover/74 grow shrink basis-full line-clamp-2 m-0">
                      {s().place}
                    </Text>
                    <Text class="text-base tabular-nums text-accent-hover/74 m-0 shrink">
                      {t("screen.save-load.Played", { s_played: formatDuration(s().played) })}
                    </Text>
                    <Text class="text-base tabular-nums text-accent-hover/74 m-0 shrink">{s().date}</Text>
                  </View>
                </>
              }>
                <Text class="text-[1.286rem] text-accent-hover/56 m-0 narrow:hidden">
                  {t("screen.save-load.empty")}
                </Text>
              </Show>
            </Show>
            )}
          </For>
          <View class={`shrink narrow:hidden`}>{props.preview?.()}</View>
          <View class={`flex-wrap justify-end mt-auto pt-3 gap-2 narrow:pt-0`}>
            <Button class="text-danger rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none grow shrink basis-[8.571rem] h-9 narrow:basis-[7.143rem]" onPress={(e) => props.onDelete?.(e)} onAction={{ details: (e) => props.onDelete?.(e) }}>
              <Text>{t("screen.save-load.delete")}</Text>
            </Button>
            <Show when={props.mode === "save"} fallback={
                <Show when={props.slots.length > 0}>
                  <Button class="text-accent rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none grow shrink basis-[8.571rem] h-9 narrow:basis-[7.143rem]" onPress={(e) => props.onLoad?.(e)}>
                    <Text>{t("screen.save-load.load")}</Text>
                  </Button>
                </Show>
            }>
              <Button class="text-accent-hover bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_1px_0px_#ffffff73] rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none grow shrink basis-[8.571rem] h-9 disabled:border-transparent active:border-transparent hover:bg-linear-to-b hover:from-[#171f30d6] hover:to-[#080c16e0] hover:bg-transparent hover:border-transparent hover:shadow-[inset_0px_2px_0px_#38e8ff] narrow:basis-[7.143rem] data-pressed:hover:border-transparent" onPress={(e) => props.onNewSlot?.(e)}>
                <Text>{t("screen.save-load.new")}</Text>
              </Button>
              <Button class="text-accent rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none grow shrink basis-[8.571rem] h-9 narrow:basis-[7.143rem]" onPress={(e) => props.onSave?.(e)}>
                <Text>{t("screen.save-load.save")}</Text>
              </Button>
            </Show>
          </View>
        </Scroll>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<SaveLoadScreenProps>> = {
  default: {"mode":"load","selected":"item-0","slots":[{"auto":true,"date":"Today 18:42","empty":false,"id":"item-0","place":"Old mill","played":15120,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"item-1","place":"Harbour","played":9800,"thumb":0,"title":"Chapter 2"},{"auto":false,"date":"","empty":true,"id":"item-2","place":"","played":0,"thumb":0,"title":""}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Back","details":"Delete"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"save-load","layer":"menu","focus":"Body/Slots/List","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
