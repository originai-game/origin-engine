import { file } from '@origingame/engine/author';
// Effekseer node 3 "Flash"
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
    18.0,
    18.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1667
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
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        1.0,
        1.0,
        0.152
      ]
    ],
    [
      0.2969,
      [
        1.0,
        1.0,
        1.0,
        0.3476
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.4143
      ]
    ],
    [
      0.6094,
      [
        1.0,
        1.0,
        1.0,
        0.395
      ]
    ],
    [
      0.7031,
      [
        1.0,
        1.0,
        1.0,
        0.3476
      ]
    ],
    [
      0.9,
      [
        1.0,
        1.0,
        1.0,
        0.152
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
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/glow"
  },
  "blend": "additive"
});
