import { file, ref } from "@origingame/engine/author";

export default file.timeline({ duration: 2, tracks: [
  { kind: "scene", keys: [{ time: 0, value: { scene: ref("base/scenes/hall") } }] },
  { kind: "camera", keys: [
    { time: 0, value: { position: [0, 2.5, 7], target: [0, 0.8, 0], fov: 45 } },
    { time: 2, ease: "power2.inOut", value: { position: [1.5, 1.5, 3.5], target: [-0.5, 1, -1], fov: 38 } },
  ] },
] });
