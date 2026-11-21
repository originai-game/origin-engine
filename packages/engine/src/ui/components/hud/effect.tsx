import { Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./effect.en.ogt";

export interface HudEffectProps {
  /** What a reader calls it (Haste). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: number;
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
 * its art inset inside it, the time still to run darkened over the art
 * and its time or stacks on a tag under it. The game owns every timer;
 * read only. The conic sweep and the tile's cut corner have no TSX
 * surface: the sweep is a top-down overlay and the corner a rounded one.
 */
export function HudEffect(props: HudEffectProps) {
  const timed = () => (props.remaining ?? -1) >= 0;
  return (
    <View
      class="flex-col items-center gap-y-[0.214rem] w-fit max-w-full text-fg"
      role="group"
      label={props.label ? t("hud.effect.label", { name: props.name, label: props.label }) : props.name}
      data-harmful={props.harmful}
    >
      <View class="relative size-[2.143rem] compact:size-[1.857rem] rounded-tl-[0.357rem] bg-[#55e39a] data-harmful:bg-[#ff4d5e]">
        <Show when={props.icon} fallback={
          <View class="absolute inset-[0.143rem] rounded-tl-[0.357rem] flex-row items-center justify-center bg-[#2a2d38]">
            <Show when={props.glyph}>
              <Icon class="size-[60%] text-fg" name={props.glyph} />
            </Show>
          </View>
        }>
          <Image class="absolute inset-[0.143rem] rounded-tl-[0.357rem]" src={String(props.icon)} />
        </Show>
        <Show when={timed()}>
          <View class="absolute left-[0.143rem] right-[0.143rem] top-[0.143rem] rounded-tl-[0.357rem] bg-[#151820ad]" style={{ height: `${Math.min(Math.max(props.remaining ?? 0, 0), 1) * 100}%` }} />
        </Show>
      </View>
      <Show when={props.label}>
        <Text class="max-w-[4.571rem] py-0 px-[0.357rem] text-[0.857rem] font-bold tabular-nums whitespace-nowrap truncate text-fg text-shadow-2xs bg-[#1b1f29e0] shadow-[inset_0px_0px_0px_1px_#ffffff38] rounded-tl-[0.286rem] compact:px-[0.214rem] data-harmful:text-[#ff4d5e]">
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
