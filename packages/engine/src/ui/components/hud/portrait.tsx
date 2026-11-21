import type { Texture } from "@origingame/engine";
import { Icon, Image, Show, View } from "@origingame/engine/ui";

export interface HudPortraitProps {
  /** A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. */
  portrait?: Texture | 0;
  /** Out of the fight: the art greyed. */
  dimmed?: boolean;
  /** Extra classes for the root, to size it (NSH style). */
  class?: string;
}

/**
 * A portrait of a HUD, a block a plate is built from: the art (or the
 * person glyph) inset in a lit edge. Its side is the HUD portrait size; a
 * plate sets it on the place it gives the portrait. Read only. The style's
 * octagon and its greying filter have no TSX surface: the edge is a
 * rounded one and the dimmed art shows dimmed.
 */
export function HudPortrait(props: HudPortraitProps) {
  return (
    <View class={`relative size-[2.714rem] rounded-[0.857rem] bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 data-dimmed:opacity-45 ${props.class ?? ""}`} data-dimmed={props.dimmed}>
      <Show when={props.portrait} fallback={
        <View class="absolute inset-[0.286rem] rounded-[0.714rem] bg-linear-to-b from-[#2a2c3a] to-[#191b28] flex-row items-center justify-center overflow-hidden">
          <Icon class="size-1/2 text-white/66" name="person" />
        </View>
      }>
        <Image class="absolute inset-[0.286rem] rounded-[0.714rem]" src={props.portrait ?? 0} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudPortraitProps>> = {
  default: {"portrait":0,"dimmed":false},
  dimmed: {"portrait":0,"dimmed":true},
};
