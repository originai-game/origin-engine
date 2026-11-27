import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1 "Shockwave"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0917
    }
  ],
  "duration": 2.2167,
  "looping": false,
  "lifetime": [
    0.6,
    0.6
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
      0.8
    ],
    [
      0.3281,
      0.8995
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
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      0.5833,
      [
        1.0,
        1.0,
        1.0,
        0.1569
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Shockwave"
  },
  "blend": "additive"
});
