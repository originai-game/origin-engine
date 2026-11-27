import { file } from '@origingame/engine/author';
// Effekseer node 3.1 "Node"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.2,
      0.2
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.1667,
    0.3333
  ],
  "speed": [
    5.5426,
    22.1702
  ],
  "size": [
    0.15,
    0.35
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2158,
        0.2158,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.2158,
        0.2158,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
