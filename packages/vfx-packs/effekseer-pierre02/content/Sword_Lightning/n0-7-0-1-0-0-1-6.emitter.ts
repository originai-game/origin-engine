import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.6 "Burst2"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.0917
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    21.0,
    21.0
  ],
  "size": [
    50.4,
    50.4
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4444
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
      0.5
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
        0.7445,
        0.8592,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.7445,
        0.8592,
        1.0,
        0.251
      ]
    ],
    [
      0.2,
      [
        0.7445,
        0.8592,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.8592,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
