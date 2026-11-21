import { For, Icon, Image, Show, Text, View, type UiEvent, t } from "@origingame/engine/ui";
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
 * buttons, one per overlay, its picture or icon beside its name. Read
 * only; the buttons report their choice. The buttons' pressed state is
 * worded, as the style words it; a pack's picture fills the button's face.
 */
export function OverlayBar(props: OverlayBarProps) {
  return (
    <View class={`flex-col items-center max-w-full gap-y-2`}>
      <For each={props.overlays ?? []} key={(o) => o.id}>
        {(o) => (
          <Show when={o().id === (props.selected ?? "")}>
            <View
              class={`flex-col w-[20rem] max-w-full gap-y-[0.286rem] border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border py-2 px-3`}
              role="status"
              label={`${o().name}: ${o().low ?? ""} – ${o().high ?? ""}`}
            >
              <Text class="text-[1rem] font-semibold whitespace-nowrap text-fg text-shadow-xs">{o().name}</Text>
              <View class="h-2 w-full rounded-[0.143rem]" style={{ background: `linear-gradient(90deg, ${o().ramp ?? ""})` }} />
              <View class="flex-row w-full gap-x-[0.857rem]">
                <Text class="min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg-muted text-shadow-xs">{o().low ?? ""}</Text>
                <Text class="min-w-0 flex-1 text-end text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg-muted text-shadow-xs">{o().high ?? ""}</Text>
              </View>
            </View>
          </Show>
        )}
      </For>
      <View class={`flex-row max-w-full gap-2 p-2 border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border`} role="toolbar" label={(props.label ?? "") !== "" ? props.label : t("component.overlay-bar.name")}>
        <For each={props.overlays ?? []} key={(o) => o.id}>
          {(o) => (
            <View
              class={`shrink-0 flex-row items-center h-14 py-1 pr-3 pl-1 gap-x-2 border-rim-strong bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-[0px_0px_0px_1px_#00000073,inset_0px_1px_0px_0px_#ffffff14,0px_6px_18px_-6px_#00000099] rounded-button border active:bg-none active:bg-tile-pressed active:scale-96`}
              role="button"
              label={o().name}
              pressed={o().id === (props.selected ?? "")}
              onPress={(e) => props.onChoose?.(e, o().id)}
            >
              <Show when={o().image} fallback={
                <View class="size-12 flex-col items-center justify-center rounded-[0.286rem] bg-tile border border-rim-soft">
                  <Show when={o().icon}>
                    <Icon class="size-6 text-fg" name={o().icon} />
                  </Show>
                </View>
              }>
                <Image class="size-12 rounded-[0.286rem]" src={o().image} />
              </Show>
              <Text class="text-[0.929rem] font-semibold whitespace-nowrap text-fg text-shadow-xs">{o().name}</Text>
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<OverlayBarProps>> = {
  default: {"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]},
  none: {"selected":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"}]},
};
