import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs off its end.
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 12,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.4,
    0.6
  ],
  "speed": [
    1.2,
    1.8
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
    0.2,
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
