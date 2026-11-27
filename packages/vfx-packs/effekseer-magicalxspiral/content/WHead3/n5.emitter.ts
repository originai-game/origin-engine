import { file } from '@origingame/engine/author';
// Effekseer node 5 "Lines"
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
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
    0.1333,
    0.1333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    4.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.1667
    ],
    [
      1.0,
      0.1667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3537,
        0.1166,
        0.251
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.3537,
        0.1166,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3537,
        0.1166,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7445,
        0.3587,
        0.502
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.7445,
        0.3587,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7445,
        0.3587,
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
