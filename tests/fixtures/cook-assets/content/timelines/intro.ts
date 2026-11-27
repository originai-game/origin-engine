import { file, ref } from "@origingame/engine/author";

export default file.timeline({
  name: "Intro", duration: 10,
  paths: { flyby: { kind: "catmullRom", points: [[-5, 3.2, 9], [-1.5, 2.4, 7.5], [2, 2, 7], [5, 2.8, 8.5]] } },
  tracks: [
    { kind: "camera", rail: "flyby", keys: [
      { time: 0, value: { at: 0, target: [0, 1, 0], fov: 50 } },
      { time: 9.5, ease: "power2.inOut", value: { at: 1, target: [1.5, 1, 0], fov: 42 } },
    ] },
    { kind: "transform", entity: "trickster", keys: [
      { time: 2, value: { position: [2, 0, 0] } },
      { time: 5, ease: "power2.inOut", value: { position: [0.5, 0, 1.5] } },
    ] },
    { kind: "animation", entity: "trickster", keys: [
      { time: 0, value: { clip: "idle", loop: true } },
      { time: 2, value: { clip: "walk", loop: true, blendIn: 0.3 } },
      { time: 5, value: { clip: "wave", blendIn: 0.25 } },
    ] },
    { kind: "caption", imports: [{ source: "assets/timelines/narration.srt" }] },
    { kind: "audio", keys: [
      { time: 1, value: { sound: ref("base/audio/blip"), volume: 0.6 } },
      { time: 5, value: { sound: ref("base/audio/click") } },
    ] },
    { kind: "music", keys: [{ time: 6, value: { sound: ref("base/audio/sting"), volume: 0.7, fadeIn: 0.3, fadeOut: 1, end: 9.5, loop: true } }] },
    { kind: "event", keys: [{ time: 5.5, value: { action: "spawn_wave:2", retroactive: true } }] },
    { kind: "transition", keys: [
      { time: 0, value: { kind: "fade", mode: "in", duration: 1 } },
      { time: 9, value: { kind: "fade", mode: "out", duration: 1 } },
    ] },
    { kind: "marker", keys: [{ time: 2, value: { name: "walk" } }, { time: 5.5, value: { name: "drop" } }] },
  ],
});
