import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1 "Hit2"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.5,
    6.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7692
    ],
    [
      0.6719,
      0.9243
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.0154
    ],
    [
      0.6719,
      0.0001
    ],
    [
      1.0,
      0.0077
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7445,
        0.3787,
        0.0066,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.3787,
        0.0066,
        1.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
