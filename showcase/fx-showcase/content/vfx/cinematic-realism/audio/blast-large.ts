import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.003,
      "decay": 1.6,
      "drive": 2.0,
      "freq": 52.0,
      "freqEnd": 22.0,
      "gain": 1.0,
      "punch": 0.6,
      "sustain": 0.15,
      "wave": "sine"
    },
    {
      "attack": 0.006,
      "decay": 2.6,
      "gain": 0.8,
      "lowpass": 650.0,
      "lpSlide": -0.4,
      "sustain": 0.25,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.16,
      "gain": 0.4,
      "highpass": 1500.0,
      "sustain": 0.015,
      "wave": "noise"
    },
    {
      "attack": 0.2,
      "decay": 1.4,
      "delay": 0.35,
      "gain": 0.12,
      "highpass": 700.0,
      "lowpass": 2600.0,
      "sustain": 0.3,
      "wave": "noise"
    }
  ]
});
