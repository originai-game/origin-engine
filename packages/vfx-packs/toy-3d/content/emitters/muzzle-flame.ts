import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A short pop of flame from a toy barrel.
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "cone",
    "angleDegrees": 12,
    "radius": 0.02
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 5
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.1,
    0.14
  ],
  "speed": [
    3,
    4.5
  ],
  "size": [
    0.16,
    0.24
  ],
  "sizeOverLife": [
    [
      0,
      0.3
    ],
    [
      0.15,
      1
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    -8,
    8
  ],
  "align": {
    "kind": "velocity",
    "stretch": 0
  },
  "angularVelocity": [
    -10,
    10
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 8,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/flame")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.7,
        1.096,
        0.072,
        1
      ]
    ],
    [
      1,
      [
        1.372,
        0.087,
        0.048,
        1
      ]
    ]
  ]
});
