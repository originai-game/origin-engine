import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A toy jet's flame cone.
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "cone",
    "angleDegrees": 3,
    "radius": 0.03
  },
  "spread": 0,
  "rate": 110,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.14,
    0.22
  ],
  "speed": [
    3,
    3.8
  ],
  "size": [
    0.24,
    0.32
  ],
  "sizeOverLife": [
    [
      0,
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
  "drag": 4,
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
