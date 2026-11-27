import { file } from "@origingame/engine/author";

// AV1, one-second keyframes/chunks; sound in Opus. Seek and loop cross
// chunks, each fetched as players reach it.
export default file.video({
  source: "assets/clips/count.mp4",
  stream: true,
  fps: 30,
  size: [192, 108],
  keyframe: 1,
  chunk: 1,
});
