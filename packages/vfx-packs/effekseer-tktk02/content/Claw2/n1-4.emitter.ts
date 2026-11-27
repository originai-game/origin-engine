import { file } from '@origingame/engine/author';
// Effekseer node 1.4 "Distortion"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.5,
    17.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.1094,
      0.3732
    ],
    [
      0.2188,
      0.5191
    ],
    [
      0.3281,
      0.6405
    ],
    [
      0.4531,
      0.7523
    ],
    [
      0.5781,
      0.8393
    ],
    [
      0.7031,
      0.9052
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
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.04
});
