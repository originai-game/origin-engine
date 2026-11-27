import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.5 "Ray"
export default file.particles({
  "maxParticles": 105,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.625,
      1.125,
      5.625
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 7,
      "interval": 0.0167
    }
  ],
  "duration": 0.1167,
  "looping": false,
  "lifetime": [
    0.8333,
    1.8333
  ],
  "speed": [
    0.1169,
    0.4677
  ],
  "size": [
    0.0,
    4.5
  ],
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      1.0,
      12.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.1875,
      [
        0.2195,
        1.0,
        0.5356,
        0.502
      ]
    ],
    [
      0.625,
      [
        0.2195,
        1.0,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
