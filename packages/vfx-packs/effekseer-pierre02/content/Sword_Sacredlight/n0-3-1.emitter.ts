import { file } from '@origingame/engine/author';
// Effekseer node 0.3.1 "Ray"
export default file.particles({
  "maxParticles": 82,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      4.0,
      4.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0417
    }
  ],
  "duration": 0.1833,
  "looping": false,
  "lifetime": [
    0.25,
    1.25
  ],
  "speed": [
    0.2078,
    0.8314
  ],
  "size": [
    0.0,
    3.2
  ],
  "heightOverLife": [
    [
      0.0,
      10.0
    ],
    [
      1.0,
      30.0
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
        0.251
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
