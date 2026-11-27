import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.0.0.1 "Distortion"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.0,
        0.0
      ],
      [
        -1.4142,
        1.4142,
        0.0
      ],
      [
        -2.0,
        0.0,
        0.0
      ],
      [
        -1.4142,
        -1.4142,
        0.0
      ],
      [
        0.0,
        -2.0,
        0.0
      ],
      [
        1.4142,
        -1.4142,
        0.0
      ],
      [
        2.0,
        0.0,
        0.0
      ],
      [
        1.4142,
        1.4142,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 0.0333
    }
  ],
  "duration": 0.1833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.4,
    4.0
  ],
  "speedOverLife": [
    [
      0.0,
      0.8422
    ],
    [
      0.4062,
      1.1507
    ],
    [
      0.8125,
      1.6807
    ],
    [
      0.8281,
      1.4195
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.3333,
    1.3333
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
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
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.0627
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/FireNoise"
  },
  "sheet": {
    "columns": 2,
    "rows": 2,
    "frames": 4,
    "mode": {
      "kind": "off"
    }
  },
  "blend": "additive",
  "distortion": 0.004
});
