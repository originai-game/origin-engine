import {Scroll, For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./film-gallery.en.ogt";

export interface FilmGalleryScreenProps {
  /** The endings, in order (og.film.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }. */
  endings: ReadonlyArray<{ id: string; title: string; thumb: Texture | 0; locked: boolean }>;
  /** The ending selected now (its key). */
  selected: string;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

export function FilmGalleryScreen(props: FilmGalleryScreenProps) {
  return (
    <View class={`pt-8 pb-6 text-fg gap-y-4 px-12 overflow-hidden compact:pt-4 compact:gap-y-3`}>
      <View class={`absolute bg-linear-to-b from-[#03060ee5] via-[#03060ed1] via-40% to-[#03060ef0] inset-0`} />
      <View class={`items-baseline gap-x-4 p-0 compact:flex-wrap compact:gap-y-1`}>
        <Button class={`text-fg/74 compact:-ml-2 ${packClass("Button", "plate")}`} onPress={(e) => props.onBack?.(e)}>
          <View class="items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
        <Text class={`font-display text-[2.857rem] font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg/60 grow shrink m-0 compact:text-[1.857rem] compact:grow compact:shrink-0 compact:basis-full`}>
          {t("screen.film-gallery.title")}
        </Text>
      </View>
      <Scroll class="grow shrink basis-0 p-4 -m-4">
        <View class="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] items-start gap-4 compact:grid-cols-[repeat(auto-fill,minmax(150px,1fr))] compact:gap-3" focus-group="wrap">
          <For each={props.endings}>
            {(c) => (
            <View class={`group/card transition duration-317 gap-y-2 p-0 data-selected:border-accent data-down:opacity-45`} onPress={(e) => props.onSelect?.(e, c().id)} disabled={c().locked} tabIndex={0} data-locked={c().locked} data-selected={c().id === props.selected}>
              <View class={`aspect-[16/9] nine-plate-cut overflow-hidden`}>
                <Show when={c().locked} fallback={
                    <Art class={`absolute object-cover inset-0 size-full`} kind="keyArt" src={c().thumb} />
                }>
                  <View class={`absolute items-center justify-center bg-[#070b14eb] inset-0 size-full`}>
                    <Icon class="text-[#ffb35c] size-6 shrink" name="lock" />
                  </View>
                </Show>
                <View class="absolute shadow-[inset_0px_0px_0px_1px_#ffffff29,inset_0px_1px_0px_#ffffff8c] inset-0 group-data-locked/card:shadow-[inset_0px_0px_0px_1px_#ffffff1f] group-data-selected/card:shadow-[inset_0px_0px_0px_2.288px_#38e8ff,inset_0px_3.424px_0px_#ffffff80]" />
              </View>
              <View class="items-baseline gap-x-2 px-0.5">
                <Show when={!c().locked}>
                  <Text class={`self-center rotate-45 bg-gold shadow-[0px_0px_9.136px_#ffcf5a] size-[0.571rem]`} />
                </Show>
                <Text class={`text-fg/74 grow shrink basis-0 text-[1.214rem] leading-snug font-semibold m-0 group-data-selected/card:text-fg group-data-locked/card:text-fg/56 compact:text-[1.071rem]`}>
                  {c().locked ? t("screen.film-gallery.locked") : c().title}
                </Text>
              </View>
            </View>
            )}
          </For>
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<FilmGalleryScreenProps>> = {
  default: {"selected":"c2","endings":[{"id":"c1","title":"Dawn","thumb":0,"locked":false},{"id":"c2","title":"The cellar","thumb":0,"locked":false},{"id":"c3","title":"Alone","thumb":0,"locked":true},{"id":"c4","title":"The fire","thumb":0,"locked":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"select":"Watch the ending again","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"film-gallery","layer":"menu","focus":"Body/Endings","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"pop"} as const;
