import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
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
    0.1667,
    0.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
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
      0.8,
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
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/smoke_01"
  },
  "sheet": {
    "columns": 4,
    "rows": 4,
    "frames": 16,
    "mode": {
      "kind": "life",
      "cycles": 0.3125
    }
  },
  "blend": "additive"
});
