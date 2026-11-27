import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0.0.1.0 "Light"
export default file.particles({
  "maxParticles": 79,
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
      "count": 9,
      "cycles": 20,
      "interval": 0.0833
    }
  ],
  "duration": 1.6,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.5981,
    10.3923
  ],
  "size": [
    4.0,
    4.0
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
        0.5356,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5356,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        0.5356,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.2195,
        1.0,
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
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
