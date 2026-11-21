import { Icon, Image, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./effect.en.ogt";

export interface HudEffectProps {
  /** What a reader calls it (Haste). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** The share of its time still to run, 0 to 1: a dark radial sweep over the art; negative is no timer. */
  remaining?: number;
  /** Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. */
  label?: string;
  /** A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). */
  harmful?: boolean;
}

/**
 * One buff or debuff of a HUD, a block an effects row is built from: a
 * small tile whose edge is its fill (green for a boon, red for a harm),
 * its art inset inside it, the time already run darkened round the art
 * and its time or stacks on a tag under it. The game owns every timer;
 * read only. The tile's cut corner has no TSX surface: it is a rounded
 * one.
 */
export function HudEffect(props: HudEffectProps) {
  const timed = () => (props.remaining ?? -1) >= 0;
  return (
    <View
      class="group/effect flex-col items-center gap-y-0.75 w-fit max-w-full text-fg"
      role="group"
      label={props.label ? t("hud.effect.label", { name: props.name, label: props.label }) : props.name}
      data-harmful={props.harmful}
    >
      <View class="relative size-7.5 compact:size-6.5 rounded-tl-control bg-boon group-data-harmful/effect:bg-foe">
        <Show when={props.icon} fallback={
          <View class="absolute inset-0.5 rounded-tl-control flex-row items-center justify-center bg-raised-lo">
            <Show when={props.glyph}>
              <Icon class="size-3/5 text-fg" name={props.glyph} />
            </Show>
          </View>
        }>
          <Image class="absolute inset-0.5 rounded-tl-control" src={props.icon} />
        </Show>
        <Show when={timed()}>
          <View class="absolute inset-0 nine-tile bg-shade-deep" wipe={1 - Math.min(Math.max(props.remaining ?? 0, 0), 1)} wipeCcw />
        </Show>
      </View>
      <Show when={props.label}>
        <Text class={`max-w-16 py-0 px-1.25 text-xs font-bold tabular-nums whitespace-nowrap truncate text-fg text-shadow-2xs nine-chip-cut bg-raised-pressed compact:px-0.75 group-data-harmful/effect:text-foe ${packClass("Chip", "frame")}`}>
          {props.label}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudEffectProps>> = {
  default: {"name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false},
  harmful: {"name":"Hex","icon":0,"glyph":"skull","remaining":0.3,"label":"4.2","harmful":true},
  untimed: {"name":"Blessed","icon":0,"glyph":"star","remaining":-1,"label":"","harmful":false},
  empty: {"name":"","icon":0,"glyph":"","remaining":-1,"label":"","harmful":false},
  overflow: {"name":"Searing brand of the Ninefold Gate","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true},
};
