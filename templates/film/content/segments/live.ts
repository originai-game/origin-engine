import { file, ref } from "@origingame/engine/author";

// A live segment has no footage: warm its scene on the deck before the cut.
export default file.timeline({
  duration: 4,
  tracks: [
    { kind: "scene", keys: [{ time: 0, value: { scene: ref("base/scenes/ending") } }] },
    { kind: "camera", keys: [
      { time: 0, value: { position: [0, 2.2, 7], target: [0, 0.8, 0], fov: 45 } },
      { time: 4, ease: "power2.inOut", value: { position: [1.5, 1.4, 3.5], target: [0.4, 0.8, 0], fov: 38 } },
    ] },
  ],
});
