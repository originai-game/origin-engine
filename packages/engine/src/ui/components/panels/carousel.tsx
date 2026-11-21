import { For, Icon, Image, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
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
 * cut arrow keys.
 */
export function Carousel(props: CarouselProps) {
  return (
    <View class={`rounded-tl-[13.712px] rounded-tr-none rounded-br-[13.712px] rounded-bl-none bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] text-accent-hover w-90 max-w-full items-center p-3 gap-2`} role="region" label={props.label || t("component.carousel.name")}>
      <View class="flex-row items-center w-full overflow-x-auto gap-3">
        <Show when={!props.pages.length}>
          <Text class={`text-caption text-accent-hover/66 p-3 m-0 shrink`} role="status">
            {t("component.carousel.empty")}
          </Text>
        </Show>
        <For each={props.pages}>
          {(page) => (
          <View class={`border-rim-soft bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-sunk text-fg transition duration-317 w-full p-0 gap-1 rounded-[6.864px] border data-selected:border-accent data-done:border-success data-active:border-accent data-active:shadow-[0px_0px_12px_#e8e8ed] active:bg-none active:bg-tile-pressed active:scale-96 hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-empty:shadow-sunk data-locked:shadow-sunk data-locked:scale-100 disabled:shadow-sunk disabled:text-fg-subtle disabled:scale-100 data-down:opacity-45`}>
            <Show when={page().image} fallback={
                <View class={`rounded-tl-[6.864px] rounded-tr-none rounded-br-[6.864px] rounded-bl-none shadow-sunk w-full h-32`} />
            }>
              <Image class={`object-cover w-full h-32 rounded-tl-[6.864px] rounded-tr-none rounded-br-[6.864px] rounded-bl-none data-muted:opacity-45`} src={page().image} />
            </Show>
            <Show when={page().title}>
              <Text class={`font-display text-[1.143rem] font-bold leading-tight tracking-[-0.01em] text-accent-hover m-0`}>
                {page().title}
              </Text>
            </Show>
            <Show when={page().body}>
              <Text class={`text-caption text-accent-hover/66`}>
                {page().body}
              </Text>
            </Show>
          </View>
          )}
        </For>
      </View>
      <View class="flex-row items-center w-full mt-2 gap-3">
        <View class={`p-0 size-11`} role="button" label={t("component.carousel.prev")} disabled={props.pages.length < 2} onPress={(e) => props.onPrev?.(e)}>
          <Icon name="back" />
        </View>
        <View class="flex-row items-center grow shrink justify-center gap-1">
          <For each={props.pages}>
            {(page) => (
            <View class={`bg-accent-hover/22 text-fg w-4 h-0.5 shrink`} />
            )}
          </For>
          <Show when={props.pages.length}>
            <View class={`bg-[#38e8ff] text-fg absolute top-0 w-4 h-0.5 shadow-[0px_0px_6.864px_#38e8ff] transition-colors duration-200`} />
          </Show>
        </View>
        <View class={`p-0 size-11`} role="button" label={t("component.carousel.next")} disabled={props.pages.length < 2} onPress={(e) => props.onNext?.(e)}>
          <Icon class="rotate-180" name="back" />
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CarouselProps>> = {
  default: {"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"prev":"Previous","next":"Next"} as const;
