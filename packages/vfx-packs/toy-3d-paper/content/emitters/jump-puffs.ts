import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs pushed out by a jump.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "cone",
    "angleDegrees": 85,
    "radius": 0.1
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 8
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.35,
    0.55
  ],
  "speed": [
    1.4,
    2
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
    0.2,
    0
  ],
  "drag": 6,
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
