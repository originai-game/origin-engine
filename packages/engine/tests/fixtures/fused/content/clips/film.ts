import { file } from "@origingame/engine/author";

// Intro from 0s, ending from 8s; a keyframe every second.
export default file.video({
  source: "assets/clips/film.mp4",
  stream: true,
  fps: 24,
  keyframe: 1,
  audio: false,
});
