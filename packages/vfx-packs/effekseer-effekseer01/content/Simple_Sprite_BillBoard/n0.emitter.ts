import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    4.3301,
    17.3205
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
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
        1.0
      ]
    ],
    [
      0.1094,
      [
        1.0,
        0.775,
        0.775,
        0.8906
      ]
    ],
    [
      0.2344,
      [
        1.0,
        0.5557,
        0.5557,
        0.7656
      ]
    ],
    [
      0.3594,
      [
        1.0,
        0.3754,
        0.3754,
        0.6406
      ]
    ],
    [
      0.4844,
      [
        1.0,
        0.2329,
        0.2329,
        0.5156
      ]
    ],
    [
      0.6094,
      [
        1.0,
        0.1264,
        0.1264,
        0.3906
      ]
    ],
    [
      0.7344,
      [
        1.0,
        0.0541,
        0.0541,
        0.2656
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
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -3.6,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle01"
  },
  "blend": "additive"
});
