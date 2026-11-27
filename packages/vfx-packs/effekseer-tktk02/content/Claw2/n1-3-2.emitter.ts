import { file } from '@origingame/engine/author';
// Effekseer node 1.3.2 "Glow3"
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
    0.6667,
    0.6667
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
      0.0667
    ],
    [
      0.0938,
      0.2404
    ],
    [
      0.2031,
      0.4104
    ],
    [
      0.3125,
      0.5496
    ],
    [
      0.4219,
      0.6623
    ],
    [
      0.6562,
      0.8357
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
        1.0,
        0.2195,
        0.0
      ]
    ],
    [
      0.125,
      [
        0.7083,
        0.8691,
        0.139,
        0.7529
      ]
    ],
    [
      0.25,
      [
        0.4704,
        0.7482,
        0.0781,
        1.0
      ]
    ],
    [
      0.375,
      [
        0.2845,
        0.6371,
        0.0356,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.148,
        0.5356,
        0.0104,
        1.0
      ]
    ],
    [
      0.75,
      [
        0.0904,
        0.3587,
        0.0023,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
