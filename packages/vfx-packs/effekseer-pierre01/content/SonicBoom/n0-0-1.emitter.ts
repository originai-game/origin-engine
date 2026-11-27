import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1 "Light1"
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
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    37.0141,
    37.0141
  ],
  "sizeOverLife": [
    [
      0.0,
      0.027
    ],
    [
      0.125,
      0.3021
    ],
    [
      0.25,
      0.5316
    ],
    [
      0.375,
      0.7154
    ],
    [
      0.5,
      0.8537
    ],
    [
      0.75,
      0.9935
    ],
    [
      0.875,
      0.9951
    ],
    [
      1.0,
      0.951
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
        0.3012
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
