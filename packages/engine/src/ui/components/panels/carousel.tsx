import { For, Icon, Image, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./carousel.en.ogt";

export interface CarouselProps {
  /** The pages: each { id, title, body, image }, image a texture handle (0: none). */
  pages: ReadonlyArray<{ id: string; title: string; body: string; image: Texture | 0 }>;
  /** The current page, from 0 (its dot lit). */
  index: number;
  /** What a screen reader calls it. */
  label: string;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The prev event, with the row's key where it comes from a row. */
  onPrev?: (e: UiEvent, key?: string) => void;
}

/**
 * Pages of art on the HUD plate: the current one ringed in the accent,
 * its neighbours set back; the place is a row of leaning pips between
 * cut arrow keys. Reports prev and next.
 *
 * Dropped from the .svelte: the pages' scroll-snap (the strip scrolls
 * without snapping to a page), the pips' lean (skew is not a class the
 * record draws), and the plate's rolled corner rims (the cut shape
 * carries no border here); the lit pip keeps its place by the index.
 */
export function Carousel(props: CarouselProps) {
  return (
    <View class={`nine-panel-cut bg-plate-hi text-fg w-90 max-w-full items-center p-3 gap-2 ${packClass("Panel", "frame")}`} role="region" label={props.label || t("component.carousel.name")}>
      <View class="flex-row items-center w-full overflow-x-auto gap-3">
        <Show when={!props.pages.length}>
          <Text class={`text-caption text-fg-muted p-3 m-0 shrink`} role="status">
            {t("component.carousel.empty")}
          </Text>
        </Show>
        <For each={props.pages} key={(page) => page.id}>
          {(page, i) => (
          <View class={`shrink-0 w-full min-w-0 gap-1 p-0 border border-transparent data-selected:border-hud-accent rounded-[6.864px] data-selected:opacity-100 opacity-80`} data-selected={i() === props.index}>
            <Show when={page().image} fallback={
                <View class={`bg-tile shadow-sunk w-full h-32 rounded-[6.864px]`} />
            }>
              <Image class={`object-cover w-full h-32 rounded-[6.864px]`} src={page().image} />
            </Show>
            <Show when={page().title}>
              <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
                {page().title}
              </Text>
            </Show>
            <Show when={page().body}>
              <Text class={`text-caption text-fg-muted`}>
                {page().body}
              </Text>
            </Show>
          </View>
          )}
        </For>
      </View>
      <View class="flex-row items-center w-full mt-2 gap-3">
        <View class={`flex-row gap-1 p-0 size-11 shrink-0 items-center justify-center ${packClass("Button", "plate")}`} role="button" label={t("component.carousel.prev")} disabled={props.pages.length < 2} onPress={(e) => props.onPrev?.(e)}>
          <Icon class="size-4" name="back" />
        </View>
        <View class="relative flex-row items-center grow shrink justify-center gap-1">
          <For each={props.pages} key={(page) => page.id}>
            {() => (
            <View class={`bg-fg/22 w-4 h-0.5 shrink-0`} />
            )}
          </For>
          <Show when={props.pages.length}>
            <View class={`bg-hud-accent absolute top-0 w-4 h-0.5 shadow-[0px_0px_6.864px_#38e8ff] transition-colors duration-200`} style={{ left: `calc(50% + ${(props.index * 20 - props.pages.length * 10 + 2).toFixed(1)}px)` }} />
          </Show>
        </View>
        <View class={`flex-row gap-1 p-0 size-11 shrink-0 items-center justify-center ${packClass("Button", "plate")}`} role="button" label={t("component.carousel.next")} disabled={props.pages.length < 2} onPress={(e) => props.onNext?.(e)}>
          <Icon class="rotate-180" name="back" />
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CarouselProps>> = {
  default: {"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]},
  empty: {"index":0,"label":"Season pass","pages":[]},
  first: {"index":0,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"prev":"Previous","next":"Next"} as const;
