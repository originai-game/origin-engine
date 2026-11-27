import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0.0 "Light"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    28.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6719,
      0.7612
    ],
    [
      0.8438,
      0.7076
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      0.6719,
      1.2837
    ],
    [
      0.8438,
      1.1229
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
        0.2195,
        0.0478,
        0.5356,
        0.0
      ]
    ],
    [
      0.0222,
      [
        0.2195,
        0.0478,
        0.5356,
        0.0471
      ]
    ],
    [
      0.5556,
      [
        0.2195,
        0.0478,
        0.5356,
        0.0471
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
