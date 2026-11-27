import { file } from '@origingame/engine/author';
// Effekseer node 0.7.1.0.0.2.0 "Light"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.45,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 11,
      "interval": 0.05
    }
  ],
  "duration": 0.5167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    204.3005,
    209.7005
  ],
  "speedOverLife": [
    [
      0.0,
      0.7043
    ],
    [
      0.5,
      1.0374
    ],
    [
      1.0,
      1.1497
    ]
  ],
  "size": [
    9.0,
    9.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.625
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.231,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.231,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        0.231,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.231,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
