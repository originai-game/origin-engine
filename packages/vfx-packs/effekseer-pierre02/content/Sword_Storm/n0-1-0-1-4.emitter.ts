import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.4 "Light"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.2667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    3.0,
    3.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
        0.8994,
        0.8994,
        0.0
      ]
    ],
    [
      0.2667,
      [
        1.0,
        0.8994,
        0.8994,
        0.0941
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.8994,
        0.8994,
        0.0941
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8994,
        0.8994,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
