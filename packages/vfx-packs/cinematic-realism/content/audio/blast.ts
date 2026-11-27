import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.002,
      "decay": 0.9,
      "drive": 1.5,
      "freq": 70.0,
      "freqEnd": 28.0,
      "gain": 0.9,
      "punch": 0.6,
      "sustain": 0.08,
      "wave": "sine"
    },
    {
      "attack": 0.004,
      "decay": 1.6,
      "gain": 0.7,
      "lowpass": 900.0,
      "lpSlide": -0.5,
      "sustain": 0.12,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.12,
      "gain": 0.45,
      "highpass": 1800.0,
      "sustain": 0.01,
      "wave": "noise"
    }
  ]
});
