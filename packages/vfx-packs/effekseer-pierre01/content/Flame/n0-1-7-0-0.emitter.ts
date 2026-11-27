import { file } from '@origingame/engine/author';
// Effekseer node 0.1.7.0.0 "Ray"
export default file.particles({
  "maxParticles": 145,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 11,
      "cycles": 4,
      "interval": 0.2
    }
  ],
  "duration": 0.6167,
  "looping": false,
  "lifetime": [
    0.1667,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.48,
    0.48
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      0.5
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3587,
        0.1166,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.3587,
        0.1166,
        0.502
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.3587,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3587,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Star"
  },
  "blend": "additive"
});
