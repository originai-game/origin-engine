import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.04,
      "gain": 0.6,
      "highpass": 2500.0,
      "sustain": 0.004,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.035,
      "freq": 950.0,
      "freqEnd": 320.0,
      "gain": 0.3,
      "sustain": 0.004,
      "wave": "sine"
    },
    {
      "attack": 0.001,
      "decay": 0.12,
      "freq": 3100.0,
      "gain": 0.08,
      "sustain": 0.01,
      "wave": "sine"
    }
  ]
});
