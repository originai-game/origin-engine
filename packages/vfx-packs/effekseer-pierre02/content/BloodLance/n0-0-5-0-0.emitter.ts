import { file } from '@origingame/engine/author';
// Effekseer node 0.0.5.0.0 "Ray"
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
    7.04,
    7.04
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0455
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2273
    ],
    [
      1.0,
      2.7727
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.2143,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ],
    [
      0.3571,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
