import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "Node"
export default file.particles({
  "maxParticles": 18,
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
      "cycles": 20,
      "interval": 0.025
    }
  ],
  "duration": 0.4917,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    12.0,
    12.0
  ],
  "size": [
    0.7,
    0.7
  ],
  "heightOverLife": [
    [
      0.0,
      0.4286
    ],
    [
      1.0,
      0.4286
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
        1.0,
        0.0,
        0.0,
        0.7529
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0,
        0.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0,
        0.7529
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.2195,
        0.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
