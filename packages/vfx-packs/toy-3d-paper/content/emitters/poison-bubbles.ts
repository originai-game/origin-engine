import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Green bubbles rising off a poisoned figure.
export default file.particles({
  "maxParticles": 23,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.35,
      0.2
    ]
  },
  "spread": 0,
  "rate": 14,
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.4,
    0.8
  ],
  "size": [
    0.07,
    0.13
  ],
  "sizeOverLife": [
    [
      0,
      0.25
    ],
    [
      0.12,
      1
    ],
    [
      0.7,
      0.9
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    -20,
    20
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "angularVelocity": [
    -30,
    30
  ],
  "gravity": [
    0,
    0.6,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/cloud")
  },
  "lit": true,
  "roughness": 0.85,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.269,
        0.5,
        0.1,
        1
      ]
    ],
    [
      1,
      [
        0.269,
        0.5,
        0.1,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.44,
        0.601,
        0.138,
        1
      ]
    ],
    [
      1,
      [
        0.44,
        0.601,
        0.138,
        1
      ]
    ]
  ]
});
