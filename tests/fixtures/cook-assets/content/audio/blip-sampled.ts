import { file } from "@origingame/engine/author";

// Expose the recording as a namespaced sound, using the native sample layer.
export default file.sfx({
  length: 0.5, volume: 1,
  layers: [{ sample: { kind: "file", source: "assets/audio/blip.wav", reference: 440 }, freq: 440,
    attack: 0, decay: 0, sustain: 0.5, sustainLevel: 1, release: 0, gain: 1 }],
});
