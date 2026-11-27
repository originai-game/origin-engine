import { file } from '@origingame/engine/author';
// Effekseer node 0 "isi"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      1.0,
      2.0
    ]
  },
  "spread": 17.0894,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.25
    }
  ],
  "duration": 1.5167,
  "looping": false,
  "lifetime": [
    0.5667,
    0.5667
  ],
  "speed": [
    21.2315,
    24.627
  ],
  "speedOverLife": [
    [
      0.0,
      0.2793
    ],
    [
      0.25,
      0.8627
    ],
    [
      0.5,
      1.183
    ],
    [
      0.75,
      1.2287
    ],
    [
      1.0,
      1.0057
    ]
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-nextsoft01/MagicMeteo/n0-1.emitter"
      }
    }
  ]
});
