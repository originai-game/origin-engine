import { file, ref } from "@origingame/engine/author";

export default file.timeline({ duration: 2, tracks: [
  { kind: "video", keys: [{ time: 0, value: { video: ref("base/clips/film"), from: 6 } }] },
] });
