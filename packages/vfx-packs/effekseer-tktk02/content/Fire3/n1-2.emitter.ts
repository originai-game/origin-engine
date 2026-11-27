import { file } from '@origingame/engine/author';
// Effekseer node 1.2 "Glow"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0667
    }
  ],
  "duration": 0.2833,
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
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.2031,
      0.6841
    ],
    [
      0.4219,
      0.8191
    ],
    [
      0.6562,
      0.912
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
        0.0478,
        0.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0356,
        0.0,
        0.2196
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.017,
        0.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0,
        0.2196
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0781,
        0.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
