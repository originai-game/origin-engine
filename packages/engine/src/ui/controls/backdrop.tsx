import type { Texture } from "@origingame/engine";
import { Show, View, packClass } from "@origingame/engine/ui";
import { Art } from "./art";

export interface BackdropProps {
  /** The key art (og.load.texture, or the project's art slot); 0: the pack's own ground (Screen.backdrop), lit and vignetted. */
  art: Texture | 0;
  /** The shade over it, for the words in front: `side` down the leading third (a main menu), `foot` along the foot (loading), `full` over all of it (pause, game over), `none`. */
  veil?: "none" | "side" | "foot" | "full" | undefined;
  /** It stands over the running game, not a picture of its own: with no art, only the veil is drawn and the game shows through it. */
  over?: boolean | undefined;
  /** Classes after its own. */
  class?: string | undefined;
}

/**
 * What a full-screen built-in stands on: the key art cropped to the screen,
 * or, without art, the pack's designed ground (Screen.backdrop with its
 * glow and vignette), never a flat colour; then the veil its words need.
 * Over the running game (a pause menu) it draws the veil alone.
 */
export function Backdrop(props: BackdropProps) {
  const veil = () => props.veil ?? "none";
  return (
    <View class={`absolute inset-0 pointer-events-none overflow-hidden ${props.class ?? ""}`}>
      <Show when={props.art !== 0} fallback={
        <Show when={!props.over}>
          <View class={`absolute inset-0 ${packClass("Screen", "backdrop")}`} />
          <View class={`absolute inset-0 ${packClass("Screen", "glow")}`} />
        </Show>
      }>
        <Art class="absolute inset-0 size-full object-cover" kind="keyArt" src={props.art} />
      </Show>
      <Show when={!props.over || props.art !== 0}>
        <View class={`absolute inset-0 ${packClass("Screen", "vignette")}`} />
      </Show>
      <Show when={veil() === "side"}>
        <View class={`absolute inset-y-0 left-0 w-3/4 portrait:inset-x-0 portrait:top-auto portrait:h-3/4 portrait:w-full compact:inset-x-0 compact:top-auto compact:h-3/4 compact:w-full ${packClass("Screen", "veil", "side")}`} />
      </Show>
      <Show when={veil() === "foot"}>
        <View class={`absolute inset-x-0 bottom-0 h-3/5 portrait:h-3/4 ${packClass("Screen", "veil", "foot")}`} />
      </Show>
      <Show when={veil() === "full"}>
        <View class={`absolute inset-0 ${packClass("Screen", "veil", "full")}`} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<BackdropProps>> = {
  default: { art: 0, veil: "none" },
  side: { art: 0, veil: "side" },
};
