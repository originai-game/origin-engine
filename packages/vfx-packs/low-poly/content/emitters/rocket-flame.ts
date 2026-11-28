import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). The flame out of a toy rocket.
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "cone",
    "angleDegrees": 8,
    "radius": 0.02
  },
  "spread": 0,
  "rate": 40,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.1,
    0.18
  ],
  "speed": [
    1.5,
    2.5
  ],
  "size": [
    0.1,
    0.16
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
  "drag": 2,
  "inheritVelocity": 0.3,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/flame")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.5,
        0.967,
        0.063,
        1
      ]
    ],
    [
      1,
      [
        1.21,
        0.077,
        0.043,
        1
      ]
    ]
  ]
});
