import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A trail of puffs behind it.
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "cone",
    "angleDegrees": 10,
    "radius": 0.03
  },
  "spread": 0,
  "rate": 26,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.6,
    1
  ],
  "speed": [
    0.4,
    0.8
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
    0.2,
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
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.789,
        0.769,
        0.741,
        1
      ]
    ],
    [
      1,
      [
        0.789,
        0.769,
        0.741,
        1
      ]
    ]
  ]
});
