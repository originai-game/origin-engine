import type { Texture } from "@origingame/engine";
import { Image } from "@origingame/engine/ui";

export interface ArtProps {
  /** The slot's kind of generated art (keyArt, logo, levelThumb, bust …); the art tools fill slots by kind. */
  kind: string;
  /** The loaded texture; 0 is none yet. */
  src: Texture | 0;
  class?: string | undefined;
  label?: string | undefined;
}

/** A slot of generated art: an image that covers its box, cropped about its centre. */
export function Art(props: ArtProps) {
  return <Image src={props.src} label={props.label} data-art={props.kind} class={`object-cover ${props.class ?? ""}`} />;
}
