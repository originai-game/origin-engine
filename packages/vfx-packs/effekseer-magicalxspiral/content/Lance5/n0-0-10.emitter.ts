import { file } from '@origingame/engine/author';
// Effekseer node 0.0.10 "Node"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
  ],
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      1.0,
      4.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      0.9397,
      -0.342
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.5356,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.5356,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line5"
  },
  "blend": "additive"
});
