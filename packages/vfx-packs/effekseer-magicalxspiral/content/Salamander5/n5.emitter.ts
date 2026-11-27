import { file } from '@origingame/engine/author';
// Effekseer node 5 "Particle1"
export default file.particles({
  "maxParticles": 38,
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
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.5,
    0.8333
  ],
  "speed": [
    1.1042,
    4.4167
  ],
  "size": [
    0.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3587,
        0.3587,
        0.502
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.3587,
        0.3587,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
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
        0.502
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.7519,
        0.7519,
        0.502
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
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
