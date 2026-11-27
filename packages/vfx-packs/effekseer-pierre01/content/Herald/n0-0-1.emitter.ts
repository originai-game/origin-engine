import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1 "Light2"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.75,
    0.75
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.75,
      0.0
    ],
    [
      1.0,
      0.3333
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
      0.5,
      [
        0.8751,
        0.7519,
        1.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        0.8751,
        0.7519,
        1.0,
        1.0
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
