import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.3,
      "gain": 0.85,
      "highpass": 250.0,
      "lowpass": 3600.0,
      "punch": 0.8,
      "sustain": 0.02,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.24,
      "drive": 2.5,
      "freq": 110.0,
      "freqEnd": 34.0,
      "gain": 0.9,
      "sustain": 0.02,
      "wave": "sine"
    },
    {
      "attack": 0.01,
      "decay": 0.95,
      "delay": 0.03,
      "gain": 0.3,
      "lowpass": 900.0,
      "sustain": 0.06,
      "wave": "noise"
    }
  ]
});
