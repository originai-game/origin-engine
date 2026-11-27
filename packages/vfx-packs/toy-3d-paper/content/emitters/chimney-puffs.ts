import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A slow string of puffs from a chimney.
export default file.particles({
  "maxParticles": 41,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.06
  },
  "spread": 0,
  "rate": 8,
  "duration": 2,
  "looping": true,
  "lifetime": [
    2.4,
    3
  ],
  "speed": [
    0.6,
    0.9
  ],
  "size": [
    0.24,
    0.34
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
      1.2
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
    0.1,
    0
  ],
  "drag": 0.4,
  "wind": 0.8,
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
  ],
  "randomColorOverLife": [
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
