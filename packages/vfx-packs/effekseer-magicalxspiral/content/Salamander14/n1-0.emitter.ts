import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "Node"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1
    ],
    [
      1.0,
      0.1
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
        0.2195,
        0.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.2195,
        0.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.2195,
        0.0,
        1.0
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
  "angularVelocity": [
    120.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
