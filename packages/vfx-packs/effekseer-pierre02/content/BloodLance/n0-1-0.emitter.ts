import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "Shockwave"
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
      "count": 1,
      "cycles": 3,
      "interval": 0.0417
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.5,
    13.5
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
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.0478,
        0.0104,
        0.1569
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0478,
        0.0104,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Shockwave"
  },
  "blend": "additive"
});
