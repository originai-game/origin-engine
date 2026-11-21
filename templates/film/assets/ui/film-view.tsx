import type { Texture } from "@origingame/engine";
import { Image, Show, View } from "@origingame/engine/ui";

export interface FilmViewProps {
  /** The texture og.film.view gives for the segment playing (0 before the first frame). */
  picture: Texture | 0;
  /** The picture dissolving out under it (FilmView.previous; 0: none). */
  previous: Texture | 0;
  /** How far the dissolve is: the picture's opacity over the previous one (FilmView.mix). */
  mix: number;
}

/** The footage fills the surface, so FilmChoices' hotspot fractions land on the picture (film §4). */
export function FilmView(props: FilmViewProps) {
  return (
    <View class="absolute inset-0 bg-shade-deep">
      <Show when={props.previous !== 0}>
        <Image class="absolute inset-0 size-full object-cover" src={props.previous} />
      </Show>
      <Image class="absolute inset-0 size-full object-cover" src={props.picture} style={{ opacity: props.previous !== 0 ? props.mix : 1 }} />
    </View>
  );
}

export const fixtures: Record<string, Partial<FilmViewProps>> = {
  default: { picture: 0, previous: 0, mix: 1 },
};
