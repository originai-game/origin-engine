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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
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
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.7969,
      [
        1.0,
        1.0,
        1.0,
        0.8905
      ]
    ],
    [
      0.875,
      [
        1.0,
        1.0,
        1.0,
        0.8415
      ]
    ],
    [
      0.9219,
      [
        1.0,
        1.0,
        1.0,
        0.716
      ]
    ],
    [
      0.9531,
      [
        1.0,
        1.0,
        1.0,
        0.5257
      ]
    ],
    [
      0.9688,
      [
        1.0,
        1.0,
        1.0,
        0.3865
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
  "angularVelocity": [
    180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/circle4"
  },
  "blend": "additive"
});
