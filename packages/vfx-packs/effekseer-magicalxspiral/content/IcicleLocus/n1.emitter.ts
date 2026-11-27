import { file } from '@origingame/engine/author';
// Effekseer node 1 "Node"
export default file.particles({
  "maxParticles": 13,
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
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.0833,
    0.25
  ],
  "speed": [
    3.8971,
    15.5885
  ],
  "size": [
    0.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.5356,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.0,
        0.5356,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.5356,
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
