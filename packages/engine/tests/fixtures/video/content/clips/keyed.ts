import { file } from "@origingame/engine/author";

// White with a horizontal alpha ramp, QuickTime Animation, one second.
// Packed alpha stays below the colour in the stream (video-alpha check).
export default file.video({
  source: "assets/clips/keyed.mov",
  stream: true,
  fps: 10,
  size: [192, 108],
  keyframe: 1,
  audio: false,
  alpha: true,
});
