import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.2 "Ray"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 8.5714,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    18.0,
    18.0
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
  "heightOverLife": [
    [
      0.0,
      0.1111
    ],
    [
      1.0,
      0.2222
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.8751,
        0.7519,
        0.0672
      ]
    ],
    [
      0.4667,
      [
        1.0,
        0.8751,
        0.7519,
        0.0753
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
