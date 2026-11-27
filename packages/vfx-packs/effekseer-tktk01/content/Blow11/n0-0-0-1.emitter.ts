import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "glow大"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.8,
    10.2
  ],
  "speedOverLife": [
    [
      0.0,
      2.5559
    ],
    [
      0.2344,
      1.64
    ],
    [
      0.4531,
      0.9806
    ],
    [
      0.625,
      0.6061
    ],
    [
      0.8125,
      0.3417
    ],
    [
      0.8281,
      0.273
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
    1.6875,
    1.6875
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1481
    ],
    [
      0.1719,
      0.75
    ],
    [
      0.3281,
      1.0278
    ],
    [
      0.3438,
      1.0159
    ],
    [
      0.8281,
      0.0329
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
        0.2158,
        0.0478,
        0.0478,
        0.098
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.0478,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.098
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
