import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0.1 "FineColor"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.5625,
    1.9688
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
    0.375,
    0.375
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.4423,
        0.1636,
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
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7445,
        0.5356,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.6888,
        0.4423,
        0.749
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.6354,
        0.3587,
        0.749
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/FireFog"
  },
  "blend": "additive"
});
