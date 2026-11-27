import { file } from '@origingame/engine/author';
// Effekseer node 3.1 "Node"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.4,
      0.4,
      0.4
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
    11.0851,
    44.3405
  ],
  "size": [
    0.3,
    0.7
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3587,
        0.0104,
        0.502
      ]
    ],
    [
      0.8667,
      [
        1.0,
        0.3587,
        0.0104,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3587,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.1166,
        0.502
      ]
    ],
    [
      0.8667,
      [
        1.0,
        0.7519,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
