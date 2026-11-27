import { file, ref } from "@origingame/engine/author";

export default file.look({
  // low-resolution-3d's monochrome-dither variant in a cold mood: the ink and paper swapped for the mood's.
  Stylise: {
    palette: ref("@palettes/1bit-cold.hex"),
  },
  ArtPalette: {
    colors: ref("@palettes/1bit-cold.hex"),
  },
});
