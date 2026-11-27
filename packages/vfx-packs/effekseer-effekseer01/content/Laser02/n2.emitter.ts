import { file } from '@origingame/engine/author';
// Effekseer node 2 "Exp"
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
    101.0,
    101.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0099
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
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      0.4219,
      [
        0.2018,
        0.2018,
        1.0,
        0.9625
      ]
    ],
    [
      0.5312,
      [
        0.1808,
        0.1808,
        1.0,
        0.9156
      ]
    ],
    [
      0.625,
      [
        0.1494,
        0.1494,
        1.0,
        0.8395
      ]
    ],
    [
      0.7344,
      [
        0.0986,
        0.0986,
        1.0,
        0.6949
      ]
    ],
    [
      0.8281,
      [
        0.0503,
        0.0503,
        1.0,
        0.5118
      ]
    ],
    [
      0.9219,
      [
        0.0117,
        0.0117,
        1.0,
        0.2636
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle02"
  },
  "blend": "additive"
});
