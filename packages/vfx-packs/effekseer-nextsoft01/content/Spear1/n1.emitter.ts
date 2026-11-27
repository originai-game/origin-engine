import { file } from '@origingame/engine/author';
// Effekseer node 1 "大きい衝撃"
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
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
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
        0.1275,
        0.7843
      ]
    ],
    [
      0.3438,
      [
        1.0,
        1.0,
        0.3027,
        0.5407
      ]
    ],
    [
      0.6094,
      [
        1.0,
        1.0,
        0.4926,
        0.3551
      ]
    ],
    [
      0.7188,
      [
        1.0,
        1.0,
        0.5974,
        0.2694
      ]
    ],
    [
      0.8281,
      [
        1.0,
        1.0,
        0.7266,
        0.1744
      ]
    ],
    [
      0.9219,
      [
        1.0,
        1.0,
        0.863,
        0.0836
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Particle01"
  },
  "blend": "additive"
});
