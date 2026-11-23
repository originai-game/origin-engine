import { For, Icon, Image, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface ShooterEffect {
  id: string;
  /** What a reader calls it (Haste). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** The share of its time still to run, 0 to 1: a dark sweep over the art; negative is no timer. */
  remaining?: number;
  /** Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. */
  label?: string;
  /** A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). */
  harmful?: boolean;
}

export interface ShooterEffectsProps {
  /** The effects on the player in the game's order. Read only. */
  effects?: ReadonlyArray<ShooterEffect>;
}

/**
 * The effects on the player, over the vitals: each a small tile with two
 * corners cut, its edge green for a boon and red for a harm, the time
 * already run as a dark sweep round its art and its time or stacks on a
 * tag under it. The game owns every timer; read only. The tiles' cut
 * corners have no TSX surface: they are rounded ones.
 */
export function ShooterEffects(props: ShooterEffectsProps) {
  return (
    <View class="flex-row flex-wrap gap-1 max-w-full text-fg" label={t("shooter.effects.name")}>
      <For each={props.effects ?? []} key={(e) => e.id}>
        {(e) => (
          <View
            class="group/effects flex-col items-center gap-y-0.75 w-fit max-w-full"
            label={e().label ? t("shooter.effects.effect", { name: e().name, label: e().label }) : e().name}
            data-harmful={e().harmful}
          >
            <View class="relative size-8 compact:size-6 rounded-tl-control bg-success group-data-harmful/effects:bg-danger">
              <Show when={e().icon} fallback={
                <View class="absolute inset-0.5 rounded-tl-control flex-row items-center justify-center bg-plate-hi">
                  <Show when={e().glyph}>
                    <Icon class="size-3/5 text-fg" name={e().glyph} />
                  </Show>
                </View>
              }>
                <Image class="absolute inset-0.5 rounded-tl-control" src={e().icon ?? 0} />
              </Show>
              <Show when={(e().remaining ?? -1) >= 0}>
                <View class="absolute inset-0 nine-tile bg-scrim" wipe={1 - Math.min(Math.max(e().remaining ?? 0, 0), 1)} wipeCcw />
              </Show>
            </View>
            <Show when={e().label}>
              <Text class={`max-w-14 py-0 px-1.25 text-xs font-bold tabular-nums whitespace-nowrap truncate text-fg text-shadow-tag nine-chip-cut bg-tile compact:px-0.75 group-data-harmful/effects:text-danger ${packClass("Chip", "frame")}`}>
                {e().label}
              </Text>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterEffectsProps>> = {
  default: {"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false},{"id":"fury","name":"Fury","icon":0,"glyph":"sword","remaining":0.35,"label":"×3 4.2","harmful":false},{"id":"burn","name":"Burning","icon":0,"glyph":"fire","remaining":0.55,"label":"6","harmful":true}]},
  untimed: {"effects":[{"id":"armor","name":"Armored","icon":0,"glyph":"shield","remaining":-1,"label":"","harmful":false}]},
  overflow: {"effects":[{"id":"e1","name":"A","icon":0,"glyph":"bolt","remaining":0.9,"label":"12:34:56","harmful":false},{"id":"e2","name":"B","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true},{"id":"e3","name":"C","icon":0,"glyph":"drop","remaining":0.5,"label":"1","harmful":true},{"id":"e4","name":"D","icon":0,"glyph":"heart","remaining":0.2,"label":"2","harmful":false},{"id":"e5","name":"E","icon":0,"glyph":"star","remaining":0.4,"label":"3","harmful":false},{"id":"e6","name":"F","icon":0,"glyph":"skull","remaining":0.6,"label":"4","harmful":true},{"id":"e7","name":"G","icon":0,"glyph":"gem","remaining":0.8,"label":"5","harmful":false},{"id":"e8","name":"H","icon":0,"glyph":"key","remaining":1,"label":"6","harmful":false}]},
};
