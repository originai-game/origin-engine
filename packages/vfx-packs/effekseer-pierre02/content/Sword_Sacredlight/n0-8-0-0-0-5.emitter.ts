import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0.0.5 "Ray"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.6875,
      0.8438,
      1.6875
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 16,
      "cycles": 3,
      "interval": 0.1167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.0,
    2.0
  ],
  "speed": [
    0.0877,
    0.3507
  ],
  "size": [
    0.1688,
    0.8438
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      13.3333
    ],
    [
      1.0,
      40.0
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
      0.1667,
      [
        1.0,
        0.8751,
        0.7519,
        0.251
      ]
    ],
    [
      0.6667,
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
