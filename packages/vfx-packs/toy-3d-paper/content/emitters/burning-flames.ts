import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Flames licking up a burning figure.
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.18,
      0.4,
      0.18
    ]
  },
  "spread": 0,
  "rate": 40,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.35,
    0.55
  ],
  "speed": [
    0.4,
    0.8
  ],
  "size": [
    0.22,
    0.34
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
    "mesh": ref("base/vfx/toy-3d-paper/models/flameFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.708,
        0.493,
        0.109,
        1
      ]
    ],
    [
      1,
      [
        0.573,
        0.116,
        0.082,
        1
      ]
    ]
  ]
});
