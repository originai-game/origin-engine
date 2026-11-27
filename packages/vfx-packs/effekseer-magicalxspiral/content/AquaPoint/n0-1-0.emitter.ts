import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "Node"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
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
    2.1667,
    2.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.125
    ],
    [
      1.0,
      0.125
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.5356,
        1.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.5356,
        1.0,
        0.7529
      ]
    ]
  ],
  "angularVelocity": [
    180.0,
    300.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
