import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Flame pieces licking up from a campfire.
export default file.particles({
  "maxParticles": 43,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 15,
  "rate": 36,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.45,
    0.7
  ],
  "speed": [
    0.3,
    0.7
  ],
  "size": [
    0.3,
    0.46
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
