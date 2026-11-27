import { file } from '@origingame/engine/author';
// Effekseer node 2.0.4 "Glow"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.1094,
      0.2165
    ],
    [
      0.2188,
      0.3989
    ],
    [
      0.3281,
      0.5506
    ],
    [
      0.4531,
      0.6904
    ],
    [
      0.5781,
      0.7991
    ],
    [
      0.7031,
      0.8815
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
        0.0104,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0254,
        0.0055,
        0.502
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0104,
        0.0023,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
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
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0478,
        0.0104,
        0.502
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0478,
        0.0104,
        0.502
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
