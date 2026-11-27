import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0.2 "Particle"
export default file.particles({
  "maxParticles": 30,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      8.0,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 12,
      "interval": 0.0333
    }
  ],
  "duration": 0.3833,
  "looping": false,
  "lifetime": [
    0.5,
    1.1667
  ],
  "speed": [
    2.5981,
    10.3923
  ],
  "size": [
    1.1,
    1.1
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0909
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
        0.9574,
        0.8751,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.8523,
        0.79,
        0.9824,
        0.9905
      ]
    ],
    [
      0.5,
      [
        0.4171,
        0.4281,
        0.8954,
        0.942
      ]
    ],
    [
      0.75,
      [
        0.208,
        0.2429,
        0.8365,
        0.4538
      ]
    ],
    [
      1.0,
      [
        0.0781,
        0.1166,
        0.7818,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9574,
        0.8751,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.9108,
        0.79,
        0.9909,
        0.9952
      ]
    ],
    [
      0.5,
      [
        0.6925,
        0.4281,
        0.945,
        0.971
      ]
    ],
    [
      0.75,
      [
        0.5576,
        0.2429,
        0.9133,
        0.4769
      ]
    ],
    [
      1.0,
      [
        0.4423,
        0.1166,
        0.8832,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.1464,
    17.9994,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
