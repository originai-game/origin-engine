import { file } from '@origingame/engine/author';
// Effekseer node 2 "LightLines"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0167
    }
  ],
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.0833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    8.0
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
        0.586,
        0.7519,
        0.0619,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.586,
        0.7519,
        0.0619,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.586,
        0.7519,
        0.0619,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9406,
        0.7519,
        0.4878,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.9406,
        0.7519,
        0.4878,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.9406,
        0.7519,
        0.4878,
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
