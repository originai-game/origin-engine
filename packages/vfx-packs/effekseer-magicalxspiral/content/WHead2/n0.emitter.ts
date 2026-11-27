import { file } from '@origingame/engine/author';
// Effekseer node 0 "Particle"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 180.0,
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
    0.1667,
    0.3333
  ],
  "speed": [
    2.5981,
    10.3923
  ],
  "size": [
    0.2,
    0.8
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.548,
        0.0478,
        0.0
      ]
    ],
    [
      0.1333,
      [
        1.0,
        0.548,
        0.0478,
        0.3765
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.548,
        0.0478,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.548,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.9914,
        0.0478,
        0.0
      ]
    ],
    [
      0.1333,
      [
        1.0,
        0.9914,
        0.0478,
        0.6275
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.9914,
        0.0478,
        0.6275
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9914,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
