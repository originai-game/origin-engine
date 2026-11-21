import {Scroll, For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./flow-gallery.en.ogt";

export interface FlowGalleryScreenProps {
  /** The endings, in order (og.flow.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }. */
  endings: ReadonlyArray<{ id: string; title: string; thumb: Texture | 0; locked: boolean }>;
  /** The ending selected now (its key). */
  selected: string;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The select event, with the row's key where it comes from a row. */
  onSelect?: (e: UiEvent, key?: string) => void;
}

export function FlowGalleryScreen(props: FlowGalleryScreenProps) {
  return (
    <View class={`pt-8 pb-6 text-fg gap-y-4 px-12 overflow-hidden compact:pt-4 compact:gap-y-3`}>
      <View class={`absolute bg-glass inset-0`} />
      <View class={`flex-row items-baseline gap-x-4 p-0 compact:flex-wrap compact:gap-y-1`}>
        <Button class={`text-fg-muted compact:-ml-2 ${packClass("Button", "plate")}`} onPress={(e) => props.onBack?.(e)}>
          <View class="flex-row items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
        <Text class={`font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg grow shrink m-0 compact:text-3xl compact:grow compact:shrink-0 compact:basis-full`}>
          {t("screen.flow-gallery.title")}
        </Text>
      </View>
      <Scroll class="grow shrink basis-0 p-4 -m-4">
        <View class="grid grid-cols-fill-47.5 items-start gap-4 compact:grid-cols-fill-37.5 compact:gap-3" focus-group="wrap">
          <For each={props.endings}>
            {(c) => (
            <View class={`group/card transition duration-317 gap-y-2 p-0 data-selected:border-accent data-down:opacity-45`} onPress={(e) => props.onSelect?.(e, c().id)} disabled={c().locked} tabIndex={0} data-locked={c().locked} data-selected={c().id === props.selected}>
              <View class={`aspect-16/9 nine-plate-cut overflow-hidden`}>
                <Show when={c().locked} fallback={
                    <Art class={`absolute object-cover inset-0 size-full`} kind="keyArt" src={c().thumb} />
                }>
                  <View class={`absolute items-center justify-center bg-shade-deep inset-0 size-full`}>
                    <Icon class="text-warn size-6 shrink" name="lock" />
                  </View>
                </Show>
                <View class="absolute inset-shadow-lit inset-ring-1 inset-ring-rim inset-0 group-data-locked/card:inset-ring-1 group-data-locked/card:inset-ring-rim group-data-selected/card:inset-shadow-lit group-data-selected/card:inset-ring-2 group-data-selected/card:inset-ring-hud-accent" />
              </View>
              <View class="flex-row items-baseline gap-x-2 px-0.5">
                <Show when={!c().locked}>
                  <Text class={`self-center rotate-45 bg-gold shadow-glow-sm shadow-gold size-2`} />
                </Show>
                <Text class={`text-fg-muted grow shrink basis-0 text-base leading-snug font-semibold m-0 group-data-selected/card:text-fg group-data-locked/card:text-fg-muted compact:text-reading`}>
                  {c().locked ? t("screen.flow-gallery.locked") : c().title}
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

export const fixtures: Record<string, Partial<FlowGalleryScreenProps>> = {
  default: {"selected":"c2","endings":[{"id":"c1","title":"Dawn","thumb":0,"locked":false},{"id":"c2","title":"The cellar","thumb":0,"locked":false},{"id":"c3","title":"Alone","thumb":0,"locked":true},{"id":"c4","title":"The fire","thumb":0,"locked":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"select":"Watch the ending again","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"flow-gallery","layer":"menu","focus":"Body/Endings","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"pop"} as const;
