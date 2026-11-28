import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.012,
      "decay": 0.6,
      "gain": 0.6,
      "highpass": 350.0,
      "lowpass": 3200.0,
      "lpSlide": -0.3,
      "sustain": 0.08,
      "wave": "noise"
    },
    {
      "attack": 0.004,
      "decay": 0.12,
      "freq": 190.0,
      "freqEnd": 85.0,
      "gain": 0.35,
      "sustain": 0.02,
      "wave": "sine"
    },
    {
      "attack": 0.05,
      "decay": 0.5,
      "delay": 0.12,
      "gain": 0.15,
      "highpass": 2800.0,
      "sustain": 0.1,
      "wave": "noise"
    }
  ]
});
