import { file } from '@origingame/engine/author';
// Effekseer node 5 "Lines"
export default file.particles({
  "maxParticles": 10,
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
    0.1167,
    0.1167
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
        1.0,
        0.3537,
        0.3537,
        0.2196
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3537,
        0.3537,
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
        0.7445,
        0.4392
      ]
    ],
    [
      1.0,
      [
        1.0,
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
