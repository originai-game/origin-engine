import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.6.0 "Ray"
export default file.particles({
  "maxParticles": 507,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 7.5,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3,
    0.6333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.44,
    2.44
  ],
  "sizeOverLife": [
    [
      0.0,
      0.082
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1639
    ],
    [
      1.0,
      2.918
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.2195,
        0.0
      ]
    ],
    [
      0.2143,
      [
        1.0,
        0.7519,
        0.2195,
        0.502
      ]
    ],
    [
      0.3571,
      [
        1.0,
        0.7519,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
