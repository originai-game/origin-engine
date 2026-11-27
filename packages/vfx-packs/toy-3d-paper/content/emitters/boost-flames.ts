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
    "mesh": ref("base/vfx/toy-3d-paper/models/flameFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.787,
        0.548,
        0.121,
        1
      ]
    ],
    [
      1,
      [
        0.637,
        0.129,
        0.092,
        1
      ]
    ]
  ]
});
