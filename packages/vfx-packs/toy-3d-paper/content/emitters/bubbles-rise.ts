import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Glossy bubbles wobbling up.
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 20,
  "rate": 10,
  "duration": 1.5,
  "looping": true,
  "lifetime": [
    1,
    1.4
  ],
  "speed": [
    0.5,
    0.8
  ],
  "size": [
    0.05,
    0.11
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
    0.2,
    0
  ],
  "drag": 1,
  "noise": {
    "strength": 0.4,
    "frequency": 2,
    "scroll": 0.4
  },
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
        0.351,
        0.593,
        0.846,
        0.8
      ]
    ],
    [
      1,
      [
        0.351,
        0.593,
        0.846,
        0.8
      ]
    ]
  ]
});
