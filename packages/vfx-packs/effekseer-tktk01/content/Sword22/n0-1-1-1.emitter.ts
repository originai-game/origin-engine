import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.1 "炎高速"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5,
    0.8333
  ],
  "speed": [
    4.5,
    8.25
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
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
        0.0,
        0.0,
        0.0,
        0.098
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.098
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
