import { file } from "@origingame/engine/author";

// A still frame standing in for the film's own clips.
export default file.video({
  source: "assets/clips/opening.mp4",
  stream: true,
  fps: 24,
  keyframe: 1,
  audio: false,
  placeholder: true,
});
