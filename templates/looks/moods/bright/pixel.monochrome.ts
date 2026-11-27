import { file, ref } from "@origingame/engine/author";

export default file.look({
  // pixel's monochrome variant in a bright mood: the four handheld tones swapped for the mood's.
  Stylise: {
    palette: ref("@palettes/handheld4-bright.hex"),
  },
  ArtPalette: {
    colors: ref("@palettes/handheld4-bright.hex"),
  },
});
