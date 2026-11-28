import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A thin string of smoke puffs.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 0,
  "rate": 2.5,
  "duration": 1,
  "looping": true,
  "lifetime": [
    1.4,
    2
  ],
  "speed": [
    0.4,
    0.6
  ],
  "size": [
    0.14,
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
      1.1
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "angularVelocity": [
    -40,
    40
  ],
  "gravity": [
    0,
    0.2,
    0
  ],
  "drag": 0.4,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.397,
        0.386,
        0.371,
        1
      ]
    ],
    [
      1,
      [
        0.397,
        0.386,
        0.371,
        1
      ]
    ]
  ]
});
