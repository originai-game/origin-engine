import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.002,
      "decay": 0.09,
      "freq": 125.0,
      "freqEnd": 58.0,
      "gain": 0.7,
      "sustain": 0.015,
      "wave": "sine"
    },
    {
      "attack": 0.002,
      "decay": 0.11,
      "gain": 0.5,
      "lowpass": 750.0,
      "sustain": 0.01,
      "wave": "noise"
    }
  ]
});
