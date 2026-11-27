import { file } from '@origingame/engine/author';
// Effekseer node 2 "LightLine1"
export default file.particles({
  "maxParticles": 14,
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
    0.05,
    0.2833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    10.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.0625
    ],
    [
      1.0,
      0.0625
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2508,
        0.0,
        0.2508,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.2508,
        0.0,
        0.2508,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2508,
        0.0,
        0.2508,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9406,
        1.0,
        0.9406,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.9406,
        1.0,
        0.9406,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.9406,
        1.0,
        0.9406,
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
