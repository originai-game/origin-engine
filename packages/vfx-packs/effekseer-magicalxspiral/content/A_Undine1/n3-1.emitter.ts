import { file } from '@origingame/engine/author';
// Effekseer node 3.1 "Node"
export default file.particles({
  "maxParticles": 20,
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
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.1667,
    0.3333
  ],
  "speed": [
    3.8105,
    15.242
  ],
  "size": [
    0.15,
    0.35
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0104,
        1.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.0,
        0.0104,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.1166,
        1.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.0478,
        0.1166,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1166,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
