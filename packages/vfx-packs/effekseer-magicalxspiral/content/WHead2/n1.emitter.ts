import { file } from '@origingame/engine/author';
// Effekseer node 1 "Particle"
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
    0.1,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.7519,
        0.0
      ]
    ],
    [
      0.1333,
      [
        1.0,
        0.7519,
        0.7519,
        0.502
      ]
    ],
    [
      0.6667,
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
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
