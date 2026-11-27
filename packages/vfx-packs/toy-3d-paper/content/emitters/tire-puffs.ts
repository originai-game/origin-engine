import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs of a spinning tyre.
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.08
  },
  "spread": 0,
  "rate": 24,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.5,
    1
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
    0.4,
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
