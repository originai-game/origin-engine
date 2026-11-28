import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.002,
      "decay": 0.06,
      "freq": 520.0,
      "freqEnd": 1250.0,
      "gain": 0.4,
      "sustain": 0.01,
      "wave": "sine"
    },
    {
      "attack": 0.002,
      "decay": 0.08,
      "gain": 0.2,
      "highpass": 1800.0,
      "lowpass": 5000.0,
      "sustain": 0.01,
      "wave": "noise"
    }
  ]
});
