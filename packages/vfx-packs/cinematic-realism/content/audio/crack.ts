import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.07,
      "gain": 0.7,
      "highpass": 300.0,
      "lowpass": 2600.0,
      "punch": 1.0,
      "sustain": 0.008,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.08,
      "freq": 260.0,
      "freqEnd": 110.0,
      "gain": 0.5,
      "sustain": 0.01,
      "wave": "sine"
    },
    {
      "attack": 0.01,
      "decay": 0.35,
      "delay": 0.05,
      "gain": 0.25,
      "lowpass": 900.0,
      "sustain": 0.05,
      "wave": "noise"
    }
  ]
});
