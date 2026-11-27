import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "ハイライト"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.1909,
    0.7637
  ],
  "size": [
    0.6,
    0.6
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.2353
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        1.0,
        0.2353
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -5.4,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/t0002"
  },
  "blend": "additive"
});
