import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Glowing bugs drifting about.
export default file.particles({
  "maxParticles": 54,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.5,
      0.6,
      1.5
    ]
  },
  "spread": 180,
  "rate": 10,
  "duration": 3,
  "looping": true,
  "lifetime": [
    2.4,
    3.2
  ],
  "speed": [
    0.05,
    0.15
  ],
  "size": [
    0.06,
    0.09
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.08,
      1
    ],
    [
      0.5,
      0.5
    ],
    [
      0.7,
      1
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "angularVelocity": [
    0,
    0
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 0.5,
  "noise": {
    "strength": 0.4,
    "frequency": 0.6,
    "scroll": 0.2
  },
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.109,
        1.825,
        0.164,
        1
      ]
    ],
    [
      1,
      [
        1.109,
        1.825,
        0.164,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        2.4,
        2.131,
        0.61,
        1
      ]
    ],
    [
      1,
      [
        2.4,
        2.131,
        0.61,
        1
      ]
    ]
  ]
});
