import { file } from '@origingame/engine/author';
// Effekseer node 0.0.15.0 "Ray"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.02,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.052,
    0.2078
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.5356,
        0.2195,
        1.0,
        0.1569
      ]
    ],
    [
      0.6,
      [
        0.5356,
        0.2195,
        1.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Star"
  },
  "blend": "additive"
});
