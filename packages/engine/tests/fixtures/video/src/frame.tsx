import { Image, Show, View } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";

/**
 * The video player's texture on a UI image over the right half of the
 * 1280×720 view; 0 shows nothing. `swatches` adds three white boxes at 25,
 * 50 and 75 % opacity on the left half (pixels 40–120, 200–280 and 360–440
 * across, 180–260 down): the host's own blend of those alphas, which a
 * keyed ramp's columns of the same alpha match.
 */
export function Frame(props: { texture: Texture | 0; swatches: boolean }) {
  return (
    <>
      <Show when={props.swatches}>
        <View class="absolute left-10 top-45 w-20 h-20 bg-white opacity-25" />
        <View class="absolute left-50 top-45 w-20 h-20 bg-white opacity-50" />
        <View class="absolute left-90 top-45 w-20 h-20 bg-white opacity-75" />
      </Show>
      <Show when={props.texture !== 0}>
        <Image class="absolute left-160 top-45 w-160 h-90" src={props.texture} label="" />
      </Show>
    </>
  );
}
