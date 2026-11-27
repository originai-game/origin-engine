import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.4.0 "Ray"
export default file.particles({
  "maxParticles": 90,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.9375,
      3.9375,
      3.9375
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    2.3333,
    2.3333
  ],
  "speed": [
    0.007,
    0.0281
  ],
  "size": [
    0.54,
    0.54
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
      0.8571,
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
      0.8571,
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
