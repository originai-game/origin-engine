import { Image, Show } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";

/** The video player's texture on a UI image over the right half of the 1280×720 view; 0 shows nothing. */
export function Frame(props: { texture: Texture | 0 }) {
  return (
    <Show when={props.texture !== 0}>
      <Image class="absolute left-[640px] top-[180px] w-[640px] h-[360px]" src={props.texture} label="" />
    </Show>
  );
}
