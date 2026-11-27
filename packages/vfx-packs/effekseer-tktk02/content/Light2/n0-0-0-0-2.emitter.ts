import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.2 "Glow"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 14,
      "interval": 0.0333
    }
  ],
  "duration": 0.45,
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
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      0.1094,
      0.5299
    ],
    [
      0.2188,
      0.6394
    ],
    [
      0.4531,
      0.8143
    ],
    [
      0.7031,
      0.9289
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
      0.25,
      [
        0.9323,
        0.6888,
        0.0078,
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
      0.25,
      [
        0.9658,
        0.7556,
        0.0356,
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
