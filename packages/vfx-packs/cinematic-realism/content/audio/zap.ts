import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.001,
      "decay": 0.16,
      "fmFreq": 90.0,
      "fmIndex": 2.0,
      "freq": 1500.0,
      "freqEnd": 180.0,
      "gain": 0.4,
      "lowpass": 5000.0,
      "sustain": 0.02,
      "wave": "saw"
    },
    {
      "attack": 0.001,
      "decay": 0.09,
      "gain": 0.3,
      "highpass": 3200.0,
      "sustain": 0.01,
      "wave": "noise"
    },
    {
      "attack": 0.002,
      "decay": 0.12,
      "freq": 90.0,
      "freqEnd": 45.0,
      "gain": 0.4,
      "sustain": 0.02,
      "wave": "sine"
    }
  ]
});
