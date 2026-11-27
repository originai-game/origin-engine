import { file } from '@origingame/engine/author';
// Effekseer node 0 "剣跡"
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
    0.8,
    0.8
  ],
  "heightOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.125,
      0.2734
    ],
    [
      0.25,
      0.9375
    ],
    [
      0.375,
      1.9922
    ],
    [
      0.5,
      3.4375
    ],
    [
      0.625,
      5.2734
    ],
    [
      0.75,
      7.5
    ],
    [
      1.0,
      13.125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1905,
        0.0415,
        1.0
      ]
    ],
    [
      0.4219,
      [
        1.0,
        0.2011,
        0.0499,
        0.9625
      ]
    ],
    [
      0.5312,
      [
        1.0,
        0.2148,
        0.0616,
        0.9156
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.2382,
        0.0834,
        0.8395
      ]
    ],
    [
      0.7344,
      [
        1.0,
        0.2862,
        0.1353,
        0.6949
      ]
    ],
    [
      0.8438,
      [
        1.0,
        0.3685,
        0.2407,
        0.4752
      ]
    ],
    [
      0.9219,
      [
        1.0,
        0.4586,
        0.3741,
        0.2636
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.586,
        0.586,
        0.0
      ]
    ]
  ],
  "rotation": [
    -45.0,
    -45.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Line01"
  },
  "blend": "additive"
});
