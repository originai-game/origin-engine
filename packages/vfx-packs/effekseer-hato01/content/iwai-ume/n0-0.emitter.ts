import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "Node"
export default file.particles({
  "maxParticles": 1478,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.15,
      0.15,
      0.15
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2100,
      "interval": 0.0167
    }
  ],
  "duration": 35.0,
  "looping": false,
  "lifetime": [
    9.6333,
    10.0333
  ],
  "speed": [
    0.0198,
    0.0793
  ],
  "size": [
    1.8,
    1.8
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.3333
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.4259,
        0.2549,
        0.0
      ]
    ],
    [
      0.0256,
      [
        1.0,
        0.4259,
        0.2549,
        1.0
      ]
    ],
    [
      0.9702,
      [
        1.0,
        0.4259,
        0.2549,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4259,
        0.2549,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.687,
        0.5234,
        0.0
      ]
    ],
    [
      0.0256,
      [
        1.0,
        0.687,
        0.5234,
        1.0
      ]
    ],
    [
      0.9702,
      [
        1.0,
        0.687,
        0.5234,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.687,
        0.5234,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-hato01/textures/particle2"
  },
  "blend": "additive"
});
