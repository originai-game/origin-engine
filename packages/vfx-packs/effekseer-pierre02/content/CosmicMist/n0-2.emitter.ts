import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "Ray"
export default file.particles({
  "maxParticles": 100,
  "shape": {
    "kind": "box",
    "halfExtents": [
      7.2,
      1.35,
      7.2
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.1333,
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
    -1.125,
    3.375
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.6
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      8.0
    ],
    [
      1.0,
      24.0
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
