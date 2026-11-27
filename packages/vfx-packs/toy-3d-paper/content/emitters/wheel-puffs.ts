import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Dust kicked up by a wheel.
export default file.particles({
  "maxParticles": 28,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.08
  },
  "spread": 0,
  "rate": 20,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.14,
    0.22
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
    0.3,
    0
  ],
  "drag": 2.5,
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
