import { file } from '@origingame/engine/author';
// Effekseer node 1.3 "Glow"
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
    0.5833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    15.0,
    15.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      0.1094,
      0.3209
    ],
    [
      0.2188,
      0.4791
    ],
    [
      0.3281,
      0.6105
    ],
    [
      0.4531,
      0.7317
    ],
    [
      0.5781,
      0.8259
    ],
    [
      0.7031,
      0.8973
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
        1.0,
        0.7445,
        0.0104,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.9228,
        0.681,
        0.0074,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.5356,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7593,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.961,
        0.7551,
        0.034,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.8671,
        0.7445,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
