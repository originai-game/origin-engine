import { For, Icon, Image, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./carousel.en.strings.ts";

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
 * The strip scrolls without snapping to a page; the lit pip follows the
 * index.
 */
export function Carousel(props: CarouselProps) {
  const active = () => {
    const n = props.pages.length;
    if (n === 0) return -1;
    const raw = Number.isFinite(props.index) ? Math.trunc(props.index) : 0;
    return Math.min(Math.max(raw, 0), n - 1);
  };
  return (
    <View class={`w-90 max-w-full items-center p-3 gap-2 ${packClass("Panel", "frame")}`} role="region" label={props.label || t("component.carousel.name")}>
      <View class="w-full overflow-hidden">
        <Show when={!props.pages.length}>
          <Text class={`text-caption text-fg-muted p-3 m-0 shrink`} role="status">
            {t("component.carousel.empty")}
          </Text>
        </Show>
        <For each={props.pages} key={(page) => page.id}>
          {(page, i) => (
          <Show when={i() === active()}>
            <View class={`w-full min-w-0 gap-1.5 p-0 border border-hud-accent rounded-field overflow-hidden transition duration-fast ease-out starting:opacity-0`} data-selected={true}>
              <Show when={page().image} fallback={
                  <View class={`bg-tile shadow-sunk w-full h-32 rounded-field`} />
              }>
                <Image class={`object-cover w-full h-32 rounded-field`} src={page().image} />
              </Show>
              <View class="px-2 pb-2 gap-0.5">
                <Show when={page().title}>
                  <Text class={`font-display text-base font-bold leading-tight text-fg m-0`}>
                    {page().title}
                  </Text>
                </Show>
                <Show when={page().body}>
                  <Text class={`text-caption text-fg-muted`}>
                    {page().body}
                  </Text>
                </Show>
              </View>
            </View>
          </Show>
          )}
        </For>
      </View>
      <View class="flex-row items-center w-full mt-2 gap-3">
        <View class={`flex-row gap-1 p-0 size-11 shrink-0 items-center justify-center ${packClass("Button", "plate")}`} role="button" label={t("component.carousel.prev")} disabled={props.pages.length < 2} onPress={(e) => props.onPrev?.(e)}>
          <Icon class="size-4" name="back" />
        </View>
        <View class="flex-row items-center grow shrink justify-center gap-1.5">
          <For each={props.pages} key={(page) => page.id}>
            {(_, i) => (
            <View class={`bg-fg-subtle w-4 h-1 rounded-capsule shrink-0 transition duration-fast data-active:bg-hud-accent data-active:shadow-glow-sm`} data-active={i() === active()} />
            )}
          </For>
        </View>
        <View class={`flex-row gap-1 p-0 size-11 shrink-0 items-center justify-center ${packClass("Button", "plate")}`} role="button" label={t("component.carousel.next")} disabled={props.pages.length < 2} onPress={(e) => props.onNext?.(e)}>
          <Icon class="size-4 rotate-180" name="back" />
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
