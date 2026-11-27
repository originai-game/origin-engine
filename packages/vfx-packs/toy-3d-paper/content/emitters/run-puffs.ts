import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little puffs kicked up by running feet.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 14,
  "duration": 0.4,
  "looping": true,
  "lifetime": [
    0.35,
    0.55
  ],
  "speed": [
    0.3,
    0.7
  ],
  "size": [
    0.1,
    0.16
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
    0.4,
    0
  ],
  "drag": 3,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/cloud")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.617,
        0.493,
        0.328,
        1
      ]
    ],
    [
      1,
      [
        0.617,
        0.493,
        0.328,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.709,
        0.593,
        0.408,
        1
      ]
    ],
    [
      1,
      [
        0.709,
        0.593,
        0.408,
        1
      ]
    ]
  ]
});
