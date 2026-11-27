import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.12 "Burst"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4,
    0.4
  ],
  "speed": [
    7.5,
    7.5
  ],
  "size": [
    22.5,
    22.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2222
    ],
    [
      1.0,
      0.6667
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
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.375,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -3.0,
    3.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Burst_2"
  },
  "blend": "additive"
});
