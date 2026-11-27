import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.22,
      "decay": 0.45,
      "gain": 0.7,
      "highpass": 120.0,
      "lowpass": 700.0,
      "lpSlide": 0.6,
      "sustain": 0.08,
      "wave": "noise"
    },
    {
      "attack": 0.2,
      "decay": 0.3,
      "freq": 60.0,
      "freqEnd": 140.0,
      "gain": 0.3,
      "sustain": 0.05,
      "wave": "sine"
    }
  ]
});
