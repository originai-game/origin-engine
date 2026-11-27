import { file, ref } from "@origingame/engine/author";

export default file.timeline({ duration: 1, tracks: [
  { kind: "scene", keys: [{ time: 0, value: { scene: ref("base/scenes/arcade") } }] },
  { kind: "camera", keys: [{ time: 0, value: { position: [0, 1.5, 8], target: [0, 1.5, 0], fov: 45 } }] },
  { kind: "marker", keys: [{ time: 0.2, value: { name: "flow.hold" } }] },
] });
