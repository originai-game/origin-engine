import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little puffs left behind a pellet.
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "point"
  },
  "spread": 0,
  "rate": 30,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.2,
    0.35
  ],
  "speed": [
    0,
    0.2
  ],
  "size": [
    0.08,
    0.12
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
    0,
    0
  ],
  "drag": 2,
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
