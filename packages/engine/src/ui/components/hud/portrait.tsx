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
    <View class={`relative size-9.5 rounded-panel bg-linear-to-b from-rim-strong via-rim via-45% to-rim-soft data-dimmed:opacity-45 ${props.class ?? ""}`} data-dimmed={props.dimmed}>
      <Show when={props.portrait} fallback={
        <View class="absolute inset-1 rounded-card bg-linear-to-b from-raised-lo to-surface flex-row items-center justify-center overflow-hidden">
          <Icon class="size-1/2 text-fg-muted" name="person" />
        </View>
      }>
        <Image class="absolute inset-1 rounded-card" src={props.portrait ?? 0} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudPortraitProps>> = {
  default: {"portrait":0,"dimmed":false},
  dimmed: {"portrait":0,"dimmed":true},
};
