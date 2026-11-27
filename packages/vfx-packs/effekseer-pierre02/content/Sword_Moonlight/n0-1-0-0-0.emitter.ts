import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0 "Light"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 2,
      "interval": 0.2667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.6667,
    8.6667
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
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0941
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2845,
        0.2195,
        0.1506
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
