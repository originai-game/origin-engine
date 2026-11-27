import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Node"
export default file.particles({
  "maxParticles": 10,
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
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.0333,
    0.2
  ],
  "speed": [
    11.8769,
    47.5077
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
        0.3587,
        0.0104,
        0.502
      ]
    ],
    [
      0.7143,
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
      0.7143,
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
