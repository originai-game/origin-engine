import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A boost's flame out of the pipe.
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.02
  },
  "spread": 0,
  "rate": 90,
  "duration": 0.9,
  "looping": false,
  "lifetime": [
    0.12,
    0.2
  ],
  "speed": [
    2.5,
    3.5
  ],
  "size": [
    0.22,
    0.3
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
  "drag": 3,
  "inheritVelocity": 0.2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/flame")
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
