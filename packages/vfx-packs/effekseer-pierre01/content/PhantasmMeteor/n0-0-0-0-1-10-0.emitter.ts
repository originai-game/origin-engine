import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.10.0 "Ray"
export default file.particles({
  "maxParticles": 300,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.1875,
      2.1875,
      2.1875
    ]
  },
  "spread": 180.0,
  "rate": 1800.0,
  "duration": 0.0267,
  "looping": false,
  "lifetime": [
    1.1667,
    1.1667
  ],
  "speed": [
    0.0045,
    0.0182
  ],
  "size": [
    0.35,
    0.35
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.1166,
        0.5295,
        0.1255
      ]
    ],
    [
      0.7143,
      [
        0.3587,
        0.1166,
        0.5295,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.1166,
        0.5295,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.632,
        0.7519,
        0.7445,
        0.6275
      ]
    ],
    [
      0.7143,
      [
        0.632,
        0.7519,
        0.7445,
        0.6275
      ]
    ],
    [
      1.0,
      [
        0.632,
        0.7519,
        0.7445,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Star"
  },
  "blend": "additive"
});
