import { file } from '@origingame/engine/author';
// Effekseer node 4 "Burst"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    46.0,
    46.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0217
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        1.0,
        1.0
      ]
    ],
    [
      0.2656,
      [
        0.0119,
        0.0119,
        1.0,
        0.7344
      ]
    ],
    [
      0.5156,
      [
        0.0511,
        0.0511,
        1.0,
        0.4844
      ]
    ],
    [
      0.7656,
      [
        0.122,
        0.122,
        1.0,
        0.2344
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Burst01"
  },
  "blend": "additive"
});
