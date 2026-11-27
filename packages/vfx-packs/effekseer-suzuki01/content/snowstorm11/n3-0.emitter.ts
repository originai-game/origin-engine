import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "particle2"
export default file.particles({
  "maxParticles": 62,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      20.0,
      20.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 60,
      "interval": 0.0833
    }
  ],
  "duration": 4.9333,
  "looping": false,
  "lifetime": [
    4.0,
    4.0
  ],
  "speed": [
    42.0,
    78.0
  ],
  "size": [
    0.0,
    0.2
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -5.4,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle02"
  },
  "blend": "additive"
});
