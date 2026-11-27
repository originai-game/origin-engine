import { file } from '@origingame/engine/author';
// Effekseer node 0 "Particle"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.2,
      0.5,
      1.2
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
  "looping": false,
  "lifetime": [
    1.1667,
    1.5
  ],
  "speed": [
    0.6,
    1.8
  ],
  "size": [
    1.0,
    1.0
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.5356,
        0.7519,
        0.0
      ]
    ],
    [
      0.025,
      [
        0.7519,
        0.5356,
        0.7519,
        0.502
      ]
    ],
    [
      0.875,
      [
        0.7519,
        0.5356,
        0.7519,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.5356,
        0.7519,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7519,
        1.0,
        0.7519,
        0.0
      ]
    ],
    [
      0.025,
      [
        0.7519,
        1.0,
        0.7519,
        0.502
      ]
    ],
    [
      0.875,
      [
        0.7519,
        1.0,
        0.7519,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7519,
        1.0,
        0.7519,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    0.72,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle4"
  },
  "blend": "additive"
});
