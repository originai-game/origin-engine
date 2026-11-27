import { file } from '@origingame/engine/author';

// MIT Origin Engine. Synthesized oscillator/noise layers; no recording.
export default file.sfx({
  "layers": [
    {
      "attack": 0.15,
      "decay": 0.5,
      "freq": 180.0,
      "freqEnd": 420.0,
      "gain": 0.35,
      "sustain": 0.2,
      "vibDepth": 0.02,
      "vibRate": 6.0,
      "wave": "sine"
    },
    {
      "attack": 0.2,
      "decay": 0.6,
      "freq": 1400.0,
      "freqEnd": 2100.0,
      "gain": 0.1,
      "sustain": 0.1,
      "wave": "sine"
    },
    {
      "attack": 0.25,
      "decay": 0.5,
      "gain": 0.08,
      "highpass": 4000.0,
      "sustain": 0.1,
      "wave": "noise"
    }
  ]
});
