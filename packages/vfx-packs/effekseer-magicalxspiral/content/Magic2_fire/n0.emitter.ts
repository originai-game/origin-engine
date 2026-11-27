import { file } from '@origingame/engine/author';
// Effekseer node 0 "Circle"
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
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0781,
      0.2636
    ],
    [
      0.1719,
      0.5118
    ],
    [
      0.2656,
      0.6949
    ],
    [
      0.375,
      0.8395
    ],
    [
      0.4688,
      0.9156
    ],
    [
      0.5781,
      0.9625
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      0.3438,
      [
        1.0,
        0.021,
        0.021,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.09,
        0.09,
        0.502
      ]
    ],
    [
      0.7188,
      [
        1.0,
        0.1062,
        0.1062,
        0.4518
      ]
    ],
    [
      0.8438,
      [
        1.0,
        0.1511,
        0.1511,
        0.4167
      ]
    ],
    [
      0.8906,
      [
        1.0,
        0.1701,
        0.1701,
        0.3678
      ]
    ],
    [
      0.9531,
      [
        1.0,
        0.1975,
        0.1975,
        0.213
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/circle1"
  },
  "blend": "additive"
});
