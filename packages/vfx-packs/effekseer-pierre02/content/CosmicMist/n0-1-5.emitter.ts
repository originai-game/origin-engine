import { file } from '@origingame/engine/author';
// Effekseer node 0.1.5 "Flash"
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
      "count": 1,
      "cycles": 3,
      "interval": 0.0417
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.35,
    0.35
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    54.0,
    54.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.25
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.0781,
      [
        0.2195,
        1.0,
        0.5356,
        0.2531
      ]
    ],
    [
      0.1429,
      [
        0.2195,
        1.0,
        0.5356,
        0.4303
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
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
