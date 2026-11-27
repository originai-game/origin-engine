import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A small blast's flame pieces.
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 9
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.14,
    0.26
  ],
  "speed": [
    1.8,
    3
  ],
  "size": [
    0.2,
    0.32
  ],
  "sizeOverLife": [
    [
      0,
      0.4
    ],
    [
      0.2,
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
  "drag": 6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/flameFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.945,
        0.848,
        0.385,
        1
      ]
    ],
    [
      1,
      [
        0.806,
        0.333,
        0.076,
        1
      ]
    ]
  ]
});
