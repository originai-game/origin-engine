import { file, ref } from "@origingame/engine/author";

export default file.timeline({ duration: 2, tracks: [
  { kind: "video", keys: [{ time: 0, value: { video: ref("base/clips/film") } }] },
  { kind: "marker", keys: [
    { time: 0.5, value: { name: "flow.open" } },
    { time: 1.5, value: { name: "flow.lock" } },
    { time: 1.8, value: { name: "flow.cut" } },
  ] },
] });
