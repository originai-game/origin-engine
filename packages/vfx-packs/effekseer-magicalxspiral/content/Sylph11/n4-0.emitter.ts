import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "Node"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
  },
  "spread": 0.0,
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
    0.3333,
    0.3333
  ],
  "speed": [
    18.0,
    18.0
  ],
  "size": [
    0.5,
    0.5
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      1.0,
      4.0
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
        0.0,
        1.0,
        0.2158,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        1.0,
        0.2158,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0,
        1.0,
        0.5295,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        1.0,
        0.5295,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line6"
  },
  "blend": "additive"
});
