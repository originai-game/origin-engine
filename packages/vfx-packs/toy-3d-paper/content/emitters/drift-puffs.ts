import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Tyre puffs.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 10,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.35,
    0.5
  ],
  "speed": [
    0.4,
    0.8
  ],
  "size": [
    0.1,
    0.15
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
        0.868,
        0.804,
        0.712,
        1
      ]
    ],
    [
      1,
      [
        0.868,
        0.804,
        0.712,
        1
      ]
    ]
  ]
});
