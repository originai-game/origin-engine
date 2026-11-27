import { file } from '@origingame/engine/author';
// Effekseer node 7 "Particle1"
export default file.particles({
  "maxParticles": 20,
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
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    2.5981,
    10.3923
  ],
  "size": [
    0.25,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.3587,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.3587,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.7519,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.7519,
        0.251
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
