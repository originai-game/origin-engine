import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "Glow"
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
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
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
        1.0,
        0.0478,
        0.0023,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.0356,
        0.0017,
        0.4532
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0254,
        0.0012,
        0.8127
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0104,
        0.0005,
        0.6255
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.0055,
        0.0003,
        0.3989
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0023,
        0.0001,
        0.2191
      ]
    ],
    [
      0.875,
      [
        1.0,
        0.0005,
        0.0,
        0.0862
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
