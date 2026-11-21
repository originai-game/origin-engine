import { For, Icon, Image, Scroll, Show, Text, View, type UiEvent, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./overlay-bar.en.ogt";

export interface OverlayBarOverlay {
  id: string;
  /** Its name (Heat). */
  name: string;
  /** An icon name shown when there is no image. */
  icon?: string;
  /** A texture handle (0: none). */
  image?: Texture | 0;
  /** The legend's gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e). */
  ramp?: string;
  /** The word at the ramp's low end. */
  low?: string;
  /** The word at the ramp's high end. */
  high?: string;
}

export interface OverlayBarProps {
  /** The map overlays, keyed rows. */
  overlays?: ReadonlyArray<OverlayBarOverlay>;
  /** The id of the overlay shown (its legend over the bar, its button lit and named); empty is none. */
  selected?: string;
  /** What a screen reader calls the bar; empty is Overlays. */
  label?: string;
  /** Show an overlay. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * A map's overlays over the scene: a legend for the selected overlay (its
 * name, its ramp low to high and the words at its ends) over a bar of
 * buttons, one per overlay, its picture or icon beside its name; the bar
 * scrolls along when the buttons run over. Read only; the buttons report
 * their choice. The buttons' pressed state is worded, as the style words
 * it; a pack's picture fills the button's face.
 */
export function OverlayBar(props: OverlayBarProps) {
  return (
    <View class={`flex-col items-center max-w-full gap-y-2`}>
      <For each={props.overlays ?? []} key={(o) => o.id}>
        {(o) => (
          <Show when={o().id === (props.selected ?? "")}>
            <View
              class={`flex-col w-70 max-w-full gap-y-1 border-rim bg-linear-to-b from-plate-hi to-plate-lo shadow-plate rounded-block border py-2 px-3`}
              role="status"
              label={`${o().name}: ${o().low ?? ""} – ${o().high ?? ""}`}
            >
              <Text class="text-sm font-semibold whitespace-nowrap text-fg text-shadow-xs">{o().name}</Text>
              <View class="h-2 w-full rounded-[0.143rem]" style={{ background: `linear-gradient(90deg, ${o().ramp ?? ""})` }} />
              <View class="flex-row w-full gap-x-3">
                <Text class="min-w-0 text-xs font-semibold tracking-label uppercase text-fg-muted text-shadow-xs">{o().low ?? ""}</Text>
                <Text class="min-w-0 flex-1 text-end text-xs font-semibold tracking-label uppercase text-fg-muted text-shadow-xs">{o().high ?? ""}</Text>
              </View>
            </View>
          </Show>
        )}
      </For>
      <Scroll class="max-w-full" role="toolbar" label={(props.label ?? "") !== "" ? props.label : t("component.overlay-bar.name")}>
        <View class="flex-row gap-2 p-2 border-rim bg-linear-to-b from-plate-hi to-plate-lo shadow-plate rounded-block border w-fit">
          <For each={props.overlays ?? []} key={(o) => o.id}>
            {(o) => (
              <View
                class={`shrink-0 flex-row items-center h-14 py-1 pr-3 pl-1 gap-x-2 border-rim-strong bg-linear-to-b from-plate-hi to-plate-lo shadow-[0px_0px_0px_1px_#00000073,inset_0px_1px_0px_0px_#ffffff14,0px_6px_18px_-6px_#00000099] rounded-button border active:bg-none active:bg-tile-pressed active:scale-96`}
                role="button"
                label={o().name}
                pressed={o().id === (props.selected ?? "")}
                onPress={(e) => props.onChoose?.(e, o().id)}
              >
                <Show when={o().image} fallback={
                  <View class="size-12 flex-col items-center justify-center rounded-control bg-tile border border-rim-soft">
                    <Show when={o().icon}>
                      <Icon class="size-6 text-fg" name={o().icon} />
                    </Show>
                  </View>
                }>
                  <Image class="size-12 rounded-control" src={o().image} />
                </Show>
                <Text class="text-caption font-semibold whitespace-nowrap text-fg text-shadow-xs">{o().name}</Text>
              </View>
            )}
          </For>
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<OverlayBarProps>> = {
  default: {"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]},
  none: {"selected":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"}]},
};
