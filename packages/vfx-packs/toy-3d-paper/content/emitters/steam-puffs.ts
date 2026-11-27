import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs of steam hissing out of a vent.
export default file.particles({
  "maxParticles": 34,
  "shape": {
    "kind": "cone",
    "angleDegrees": 8,
    "radius": 0.04
  },
  "spread": 0,
  "rate": 24,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    2.2,
    3
  ],
  "size": [
    0.12,
    0.2
  ],
  "sizeOverLife": [
    [
      0,
      0.3
    ],
    [
      0.15,
      0.8
    ],
    [
      0.7,
      1.4
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
    0.5,
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
