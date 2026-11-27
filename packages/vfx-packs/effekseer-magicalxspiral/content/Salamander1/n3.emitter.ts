import { file } from '@origingame/engine/author';
// Effekseer node 3 "Particle1"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.05,
      0.05,
      0.05
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
    0.2167,
    0.2833
  ],
  "speed": [
    2.6847,
    10.7387
  ],
  "size": [
    0.1,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.3587,
        0.3587,
        0.7529
      ]
    ],
    [
      0.3333,
      [
        0.5356,
        0.3587,
        0.3587,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.3587,
        0.3587,
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
        0.7519,
        0.7529
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.7519,
        0.7519,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.7519,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
