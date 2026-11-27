import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs popping out of a kart's exhaust.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "cone",
    "angleDegrees": 10,
    "radius": 0.02
  },
  "spread": 0,
  "rate": 18,
  "duration": 0.6,
  "looping": true,
  "lifetime": [
    0.4,
    0.65
  ],
  "speed": [
    0.8,
    1.2
  ],
  "size": [
    0.08,
    0.13
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
    0.6,
    0
  ],
  "drag": 3,
  "wind": 0.6,
  "inheritVelocity": 0.2,
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
        0.407,
        0.4,
        0.39,
        1
      ]
    ],
    [
      1,
      [
        0.407,
        0.4,
        0.39,
        1
      ]
    ]
  ]
});
