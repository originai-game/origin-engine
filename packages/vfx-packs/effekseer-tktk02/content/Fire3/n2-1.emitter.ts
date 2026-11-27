import { file } from '@origingame/engine/author';
// Effekseer node 2.1 "Glow"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 4
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      0.0938,
      0.4574
    ],
    [
      0.2031,
      0.5789
    ],
    [
      0.4219,
      0.7588
    ],
    [
      0.6562,
      0.8827
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
      0.2,
      [
        1.0,
        0.0379,
        0.0,
        0.2259
      ]
    ],
    [
      0.8,
      [
        1.0,
        0.0155,
        0.0,
        0.1506
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
      0.0938,
      [
        1.0,
        0.1975,
        0.0,
        0.2188
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.1741,
        0.0,
        0.4267
      ]
    ],
    [
      0.8,
      [
        1.0,
        0.0714,
        0.0,
        0.2008
      ]
    ],
    [
      0.9062,
      [
        1.0,
        0.0582,
        0.0,
        0.0754
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
