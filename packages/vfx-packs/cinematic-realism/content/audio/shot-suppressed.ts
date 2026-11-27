import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.002,
      "decay": 0.09,
      "gain": 0.4,
      "highpass": 300.0,
      "lowpass": 2200.0,
      "sustain": 0.01,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.03,
      "freq": 230.0,
      "freqEnd": 120.0,
      "gain": 0.2,
      "sustain": 0.004,
      "wave": "sine"
    },
    {
      "attack": 0.001,
      "decay": 0.02,
      "delay": 0.015,
      "gain": 0.15,
      "highpass": 3500.0,
      "sustain": 0.002,
      "wave": "noise"
    }
  ]
});
