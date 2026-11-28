import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.25,
      "gain": 0.6,
      "highpass": 3800.0,
      "punch": 0.8,
      "sustain": 0.02,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.09,
      "delay": 0.04,
      "freq": 4200.0,
      "gain": 0.15,
      "sustain": 0.005,
      "wave": "sine"
    },
    {
      "attack": 0.001,
      "decay": 0.07,
      "delay": 0.11,
      "freq": 5300.0,
      "gain": 0.12,
      "sustain": 0.005,
      "wave": "sine"
    },
    {
      "attack": 0.001,
      "decay": 0.1,
      "delay": 0.19,
      "freq": 3400.0,
      "gain": 0.1,
      "sustain": 0.005,
      "wave": "sine"
    },
    {
      "attack": 0.02,
      "decay": 0.4,
      "delay": 0.1,
      "gain": 0.15,
      "highpass": 5000.0,
      "sustain": 0.1,
      "wave": "noise"
    }
  ]
});
