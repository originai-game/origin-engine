import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.2 "Flash"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    6.113,
    6.113
  ],
  "speedOverLife": [
    [
      0.0,
      0.9621
    ],
    [
      0.5781,
      0.9621
    ],
    [
      0.6094,
      1.0784
    ],
    [
      1.0,
      1.0784
    ]
  ],
  "size": [
    40.0,
    40.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.7488
    ],
    [
      0.4531,
      0.5233
    ],
    [
      0.7031,
      0.323
    ],
    [
      1.0,
      0.125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        1.0,
        1.0,
        0.1882
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.1882
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
