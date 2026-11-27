import { file } from '@origingame/engine/author';
// Effekseer node 6.0.1 "Inner Symbol"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.95,
        0.0
      ],
      [
        -0.6718,
        0.6718,
        0.0
      ],
      [
        -0.95,
        0.0,
        0.0
      ],
      [
        -0.6718,
        -0.6718,
        0.0
      ],
      [
        0.0,
        -0.95,
        0.0
      ],
      [
        0.6718,
        -0.6718,
        0.0
      ],
      [
        0.95,
        0.0,
        0.0
      ],
      [
        0.6718,
        0.6718,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0267,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.175,
    0.175
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/starpart_"
  },
  "blend": "additive"
});
