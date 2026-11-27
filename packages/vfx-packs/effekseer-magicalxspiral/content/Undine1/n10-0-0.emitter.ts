import { file } from '@origingame/engine/author';
// Effekseer node 10.0.0 "Node"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.2,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
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
  "gravity": [
    0.0,
    36.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle4"
  },
  "blend": "additive"
});
