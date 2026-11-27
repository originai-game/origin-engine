import { file } from '@origingame/engine/author';
// Effekseer node 2.1 "Node"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.4
    ],
    [
      1.0,
      0.4
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      0.4219,
      [
        0.2546,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      0.8333,
      [
        0.4469,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5418,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      0.4219,
      [
        0.5865,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      0.8333,
      [
        0.8685,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -90.0,
    -90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Thunder2"
  },
  "sheet": {
    "columns": 2,
    "rows": 4,
    "frames": 8,
    "mode": {
      "kind": "fps",
      "fps": 30.0
    }
  },
  "blend": "additive"
});
