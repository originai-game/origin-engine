import { file } from '@origingame/engine/author';
// Effekseer node 1 "Action Lines"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.5,
        0.0
      ],
      [
        -1.7678,
        1.7678,
        0.0
      ],
      [
        -2.5,
        0.0,
        0.0
      ],
      [
        -1.7678,
        -1.7678,
        0.0
      ],
      [
        0.0,
        -2.5,
        0.0
      ],
      [
        1.7678,
        -1.7678,
        0.0
      ],
      [
        2.5,
        0.0,
        0.0
      ],
      [
        1.7678,
        1.7678,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 50
    }
  ],
  "duration": 0.0167,
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
    0.25,
    0.65
  ],
  "heightOverLife": [
    [
      0.0,
      25.5556
    ],
    [
      1.0,
      25.5556
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.249
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.249
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
