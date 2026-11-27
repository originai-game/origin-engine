import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Coloured puffs of the spell.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 8
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4,
    0.7
  ],
  "speed": [
    1.2,
    2
  ],
  "size": [
    0.2,
    0.3
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
        0.513,
        0.351,
        0.846,
        1
      ]
    ],
    [
      1,
      [
        0.513,
        0.351,
        0.846,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.866,
        0.415,
        0.545,
        1
      ]
    ],
    [
      1,
      [
        0.866,
        0.415,
        0.545,
        1
      ]
    ]
  ]
});
