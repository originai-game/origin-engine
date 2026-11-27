import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "２段目衝撃"
export default file.particles({
  "maxParticles": 94,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 75
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.25,
    1.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2084,
        0.2084,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.2084,
        0.2084,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2084,
        0.2084,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.4368,
        0.4368,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.4368,
        0.4368,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4368,
        0.4368,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
