import { file, ref } from "@origingame/engine/author";

export default file.timeline({
  duration: 4,
  tracks: [
    { kind: "video", keys: [{ time: 0, value: { video: ref("base/clips/opening") } }] },
    { kind: "marker", keys: [
      { time: 1, value: { name: "flow.open" } },
      { time: 3, value: { name: "flow.lock" } },
      { time: 3.5, value: { name: "flow.cut" } },
    ] },
  ],
});
