import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.8 "Flash"
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
    0.4167,
    0.4167
  ],
  "speed": [
    6.113,
    6.113
  ],
  "speedOverLife": [
    [
      0.0,
      0.9744
    ],
    [
      0.7031,
      0.9744
    ],
    [
      0.7344,
      1.0922
    ],
    [
      1.0,
      1.0922
    ]
  ],
  "size": [
    11.875,
    11.875
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2105
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
        0.0
      ]
    ],
    [
      0.28,
      [
        1.0,
        1.0,
        1.0,
        0.875
      ]
    ],
    [
      0.32,
      [
        1.0,
        1.0,
        1.0,
        0.9444
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
