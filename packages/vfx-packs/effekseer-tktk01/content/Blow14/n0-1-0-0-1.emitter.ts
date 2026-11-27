import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.1 "glow"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
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
    0.9,
    2.1
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
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.2969,
      0.4638
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
        0.5295,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.0,
        0.0,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        0.2158,
        0.2158,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.2158,
        0.2158,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
