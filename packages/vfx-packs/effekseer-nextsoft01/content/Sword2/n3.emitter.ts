import { file } from '@origingame/engine/author';
// Effekseer node 3 "パーティクル"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "point"
  },
  "spread": 90.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.4833,
    0.5167
  ],
  "speed": [
    6.7882,
    12.7562
  ],
  "size": [
    0.6,
    0.6
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.3438,
      0.8827
    ],
    [
      0.5781,
      0.7588
    ],
    [
      0.7969,
      0.5789
    ],
    [
      0.9062,
      0.4574
    ],
    [
      1.0,
      0.3333
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      0.4375,
      [
        0.0009,
        0.2402,
        1.0,
        1.0
      ]
    ],
    [
      0.6562,
      [
        0.0274,
        0.324,
        1.0,
        1.0
      ]
    ],
    [
      0.7812,
      [
        0.1254,
        0.4502,
        1.0,
        1.0
      ]
    ],
    [
      0.8667,
      [
        0.3037,
        0.5982,
        1.0,
        1.0
      ]
    ],
    [
      0.9375,
      [
        0.5867,
        0.7792,
        1.0,
        0.4688
      ]
    ],
    [
      0.9688,
      [
        0.7699,
        0.8813,
        1.0,
        0.2344
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
  "randomColorOverLife": [
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
      0.8667,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
  "gravity": [
    0.0,
    -14.4,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/t0002"
  },
  "blend": "alpha"
});
