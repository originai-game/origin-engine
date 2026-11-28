import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Flakes drifting off it.
export default file.particles({
  "maxParticles": 18,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3,
      0.1,
      0.3
    ]
  },
  "spread": 180,
  "rate": 8,
  "duration": 1.5,
  "looping": true,
  "lifetime": [
    0.8,
    1.2
  ],
  "speed": [
    0.1,
    0.3
  ],
  "size": [
    0.04,
    0.06
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
    -0.4,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.965,
        0.93,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.965,
        0.93,
        0.871,
        1
      ]
    ]
  ]
});
