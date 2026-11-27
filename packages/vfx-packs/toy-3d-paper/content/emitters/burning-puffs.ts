import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little sooty puffs.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
  },
  "spread": 0,
  "rate": 6,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.6,
    1
  ],
  "size": [
    0.12,
    0.2
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
    1,
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
