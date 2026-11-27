import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 89,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      0.001,
      5.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 100,
      "interval": 0.0167
    }
  ],
  "duration": 1.6667,
  "looping": false,
  "lifetime": [
    1.1667,
    1.1667
  ],
  "speed": [
    0.2121,
    0.8485
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      16.0
    ],
    [
      1.0,
      16.0
    ]
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
        0.2195,
        0.2195,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
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
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle01"
  },
  "blend": "additive"
});
