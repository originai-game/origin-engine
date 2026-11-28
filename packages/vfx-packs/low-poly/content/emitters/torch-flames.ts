import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A torch's small flame.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "sphere",
    "radius": 0.04
  },
  "spread": 10,
  "rate": 26,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.3,
    0.45
  ],
  "speed": [
    0.3,
    0.6
  ],
  "size": [
    0.17,
    0.26
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
  "angularVelocity": [
    -10,
    10
  ],
  "gravity": [
    0,
    1,
    0
  ],
  "drag": 1.5,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/flame")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.35,
        0.87,
        0.057,
        1
      ]
    ],
    [
      1,
      [
        1.089,
        0.069,
        0.038,
        1
      ]
    ]
  ]
});
