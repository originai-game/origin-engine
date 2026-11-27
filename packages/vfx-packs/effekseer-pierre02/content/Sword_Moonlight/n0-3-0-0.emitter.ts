import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.0 "Flash1"
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
    64.0,
    64.0
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
      0.0833,
      [
        0.2195,
        1.0,
        0.5356,
        0.7529
      ]
    ],
    [
      0.6667,
      [
        0.2195,
        1.0,
        0.5356,
        0.7529
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
