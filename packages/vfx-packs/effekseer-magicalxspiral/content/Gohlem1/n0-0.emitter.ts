import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "Node"
export default file.particles({
  "maxParticles": 20,
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
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.1667,
    0.3333
  ],
  "speed": [
    5.5426,
    22.1702
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
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.8667,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
