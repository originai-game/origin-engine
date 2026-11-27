import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A dull cloud over its head.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 6
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.3,
    0.6
  ],
  "size": [
    0.16,
    0.26
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
    -0.3,
    0
  ],
  "drag": 3.5,
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
        0.311,
        0.178,
        0.639,
        1
      ]
    ],
    [
      1,
      [
        0.311,
        0.178,
        0.639,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.121,
        0.112,
        0.106,
        1
      ]
    ],
    [
      1,
      [
        0.121,
        0.112,
        0.106,
        1
      ]
    ]
  ]
});
