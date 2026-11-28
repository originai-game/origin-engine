import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.002,
      "decay": 0.45,
      "drive": 1.5,
      "freq": 95.0,
      "freqEnd": 38.0,
      "gain": 0.85,
      "punch": 0.6,
      "sustain": 0.04,
      "wave": "sine"
    },
    {
      "attack": 0.003,
      "decay": 0.8,
      "gain": 0.6,
      "lowpass": 1400.0,
      "lpSlide": -0.5,
      "sustain": 0.06,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.09,
      "gain": 0.5,
      "highpass": 2200.0,
      "sustain": 0.008,
      "wave": "noise"
    }
  ]
});
