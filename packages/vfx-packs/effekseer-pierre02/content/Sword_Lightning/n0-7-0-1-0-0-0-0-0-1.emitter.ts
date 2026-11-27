import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.0.0.0.1 "Light"
export default file.particles({
  "maxParticles": 97,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 12.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    0.5,
    0.5
  ],
  "speedOverLife": [
    [
      0.0,
      17.5
    ],
    [
      0.3906,
      17.5
    ],
    [
      0.4062,
      11.75
    ],
    [
      0.4219,
      15.0
    ],
    [
      0.7812,
      15.0
    ],
    [
      0.7969,
      10.5
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    4.0,
    12.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6719,
      0.6419
    ],
    [
      0.8438,
      0.5615
    ],
    [
      1.0,
      0.5
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      0.6719,
      3.2837
    ],
    [
      0.8438,
      3.1229
    ],
    [
      1.0,
      3.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0461,
        0.0759,
        0.5356,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.0461,
        0.0759,
        0.5356,
        0.502
      ]
    ],
    [
      0.3333,
      [
        0.0461,
        0.0759,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0461,
        0.0759,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
