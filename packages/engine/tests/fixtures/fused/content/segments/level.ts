import { file, ref } from "@origingame/engine/author";

export default file.timeline({ duration: 1, tracks: [
  { kind: "scene", keys: [{ time: 0, value: { scene: ref("base/scenes/level") } }] },
  { kind: "camera", keys: [{ time: 0, value: { position: [0, 3, 7], target: [0, 0.3, 0], fov: 50 } }] },
  { kind: "marker", keys: [{ time: 0.5, value: { name: "flow.hold" } }] },
] });
