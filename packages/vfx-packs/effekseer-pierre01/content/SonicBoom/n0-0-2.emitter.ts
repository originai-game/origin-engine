import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Light2"
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
    0.2,
    0.2
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    32.0,
    32.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6562,
      0.0156
    ],
    [
      0.6719,
      0.0078
    ],
    [
      1.0,
      0.5
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
      0.0781,
      [
        1.0,
        1.0,
        1.0,
        0.2161
      ]
    ],
    [
      0.1719,
      [
        1.0,
        1.0,
        1.0,
        0.427
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.5625
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.6667
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
