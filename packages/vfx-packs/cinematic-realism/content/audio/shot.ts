import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.16,
      "gain": 0.8,
      "highpass": 500.0,
      "lowpass": 5200.0,
      "punch": 0.8,
      "sustain": 0.012,
      "wave": "noise"
    },
    {
      "attack": 0.001,
      "decay": 0.1,
      "drive": 2.0,
      "freq": 160.0,
      "freqEnd": 50.0,
      "gain": 0.7,
      "sustain": 0.01,
      "wave": "sine"
    },
    {
      "attack": 0.005,
      "decay": 0.45,
      "delay": 0.02,
      "gain": 0.25,
      "lowpass": 1400.0,
      "sustain": 0.03,
      "wave": "noise"
    }
  ]
});
