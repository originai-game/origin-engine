import { file } from '@origingame/engine/author';
// Effekseer node 7.0 "〆つぶつぶ"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.2667,
    0.5
  ],
  "speed": [
    10.4348,
    20.8696
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
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1562,
      0.9252
    ],
    [
      0.3281,
      0.7803
    ],
    [
      0.4688,
      0.6281
    ],
    [
      0.8281,
      0.1898
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
        0.0,
        0.6187,
        0.0,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.6187,
        0.0,
        0.5882
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.7969,
        0.2158,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.7969,
        0.2158,
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
