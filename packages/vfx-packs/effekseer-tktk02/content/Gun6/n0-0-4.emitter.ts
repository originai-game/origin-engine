import { file } from '@origingame/engine/author';
// Effekseer node 0.0.4 "Glow"
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
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.0,
    7.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1429
    ],
    [
      0.0781,
      0.3688
    ],
    [
      0.1719,
      0.5816
    ],
    [
      0.2656,
      0.7385
    ],
    [
      0.375,
      0.8625
    ],
    [
      0.4688,
      0.9277
    ],
    [
      0.5781,
      0.9678
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.2011,
        0.0438,
        0.2161
      ]
    ],
    [
      0.1719,
      [
        1.0,
        0.1801,
        0.0392,
        0.427
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.5625
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.147,
        0.032,
        0.6667
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
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
