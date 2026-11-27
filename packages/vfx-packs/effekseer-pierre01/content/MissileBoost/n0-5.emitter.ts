import { file } from '@origingame/engine/author';
// Effekseer node 0.5 "Ray"
export default file.particles({
  "maxParticles": 6,
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
    26.8,
    26.8
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3731
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1866
    ],
    [
      1.0,
      0.3119
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ],
    [
      0.1786,
      [
        1.0,
        0.1636,
        0.0254,
        1.0
      ]
    ],
    [
      0.4643,
      [
        1.0,
        0.1636,
        0.0254,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Star"
  },
  "blend": "additive"
});
