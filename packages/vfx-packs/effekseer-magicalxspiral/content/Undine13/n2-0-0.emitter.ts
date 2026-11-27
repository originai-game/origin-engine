import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0 "Node"
export default file.particles({
  "maxParticles": 28,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.2,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    1.8187,
    7.2746
  ],
  "size": [
    0.5,
    0.5
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.1636,
        0.6388,
        1.0,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.1636,
        0.6388,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.1636,
        0.6388,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2845,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.2845,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2845,
        0.8751,
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
