import { file, ref } from "@origingame/engine/author";

export default file.look({
  // low-resolution-3d's monochrome-dither variant in a warm mood: the ink and paper swapped for the mood's.
  Stylise: {
    palette: ref("@palettes/1bit-warm.hex"),
  },
  ArtPalette: {
    colors: ref("@palettes/1bit-warm.hex"),
  },
});
