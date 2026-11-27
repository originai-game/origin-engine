import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.3 "Ray"
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
    48.0,
    48.0
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
      0.0833
    ],
    [
      1.0,
      0.1667
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
        0.1793
      ]
    ],
    [
      0.4667,
      [
        1.0,
        0.8751,
        0.7519,
        0.2008
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
