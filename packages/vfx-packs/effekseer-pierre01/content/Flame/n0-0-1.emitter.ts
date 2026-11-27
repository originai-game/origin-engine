import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1 "Light"
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
    2.2667,
    2.2667
  ],
  "speed": [
    0.6,
    0.6
  ],
  "size": [
    12.0,
    12.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      0.3281,
      0.5225
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
        0.5356,
        0.1636,
        0.0
      ]
    ],
    [
      0.0938,
      [
        1.0,
        0.5356,
        0.1636,
        0.0712
      ]
    ],
    [
      0.3281,
      [
        1.0,
        0.5356,
        0.1636,
        0.1184
      ]
    ],
    [
      0.4375,
      [
        1.0,
        0.5356,
        0.1636,
        0.1931
      ]
    ],
    [
      0.5625,
      [
        1.0,
        0.5356,
        0.1636,
        0.4171
      ]
    ],
    [
      0.6618,
      [
        1.0,
        0.5356,
        0.1636,
        0.7529
      ]
    ],
    [
      0.7794,
      [
        1.0,
        0.5356,
        0.1636,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1636,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
