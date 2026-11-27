import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A small blast's puffs.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 10
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    1
  ],
  "speed": [
    1.6,
    3
  ],
  "size": [
    0.26,
    0.42
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
        0.719,
        0.297,
        0.068,
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
