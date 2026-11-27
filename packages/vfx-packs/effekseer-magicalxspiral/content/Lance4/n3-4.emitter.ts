import { file } from '@origingame/engine/author';
// Effekseer node 3.4 "Lines"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
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
    0.0833,
    0.0833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    6.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.125
    ],
    [
      1.0,
      0.125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.3537,
        0.3636,
        0.251
      ]
    ],
    [
      0.8,
      [
        0.1166,
        0.3537,
        0.3636,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.3537,
        0.3636,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.7445,
        0.7445,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.3587,
        0.7445,
        0.7445,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.7445,
        0.7445,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -120.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave3"
  },
  "blend": "additive"
});
