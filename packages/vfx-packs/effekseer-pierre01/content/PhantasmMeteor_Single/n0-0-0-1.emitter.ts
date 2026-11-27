import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Light"
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
      "cycles": 2,
      "interval": 0.3333
    }
  ],
  "duration": 0.35,
  "looping": false,
  "lifetime": [
    2.0,
    2.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    11.25,
    11.25
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1111
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
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.0833,
      [
        0.8751,
        0.7519,
        1.0,
        0.6275
      ]
    ],
    [
      0.7917,
      [
        0.8751,
        0.7519,
        1.0,
        0.6275
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
