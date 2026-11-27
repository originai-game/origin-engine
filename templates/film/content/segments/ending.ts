import { file, ref } from "@origingame/engine/author";

export default file.timeline({
  duration: 4,
  tracks: [{ kind: "video", keys: [{ time: 0, value: { video: ref("base/clips/opening") } }] }],
});
