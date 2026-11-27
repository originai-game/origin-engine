import { file } from '@origingame/engine/author';
// Effekseer node 0 "Particle1"
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
  "spread": 90.0,
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
    0.5,
    0.5
  ],
  "speed": [
    1.9799,
    3.4293
  ],
  "speedOverLife": [
    [
      0.0,
      1.0041
    ],
    [
      0.25,
      1.1667
    ],
    [
      0.5,
      1.1334
    ],
    [
      0.75,
      0.8999
    ],
    [
      1.0,
      0.4746
    ]
  ],
  "size": [
    0.3,
    0.3
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.1139,
        0.7445,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0,
        0.1139,
        0.7445,
        0.251
      ]
    ],
    [
      0.6667,
      [
        0.0,
        0.1139,
        0.7445,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.1139,
        0.7445,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0104,
        0.7445,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0104,
        0.7445,
        1.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.0104,
        0.7445,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.7445,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
