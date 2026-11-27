import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). The rocket's spark trail.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "point"
  },
  "spread": 180,
  "rate": 40,
  "duration": 0.7,
  "looping": false,
  "lifetime": [
    0.2,
    0.3
  ],
  "speed": [
    0,
    0.3
  ],
  "size": [
    0.05,
    0.07
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.8,
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
    -360,
    360
  ],
  "gravity": [
    0,
    -1,
    0
  ],
  "drag": 4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        2,
        1.776,
        0.508,
        1
      ]
    ],
    [
      1,
      [
        2,
        1.776,
        0.508,
        1
      ]
    ]
  ]
});
