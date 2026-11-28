import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). The hot heart of a toy blast: fat flame pieces thrown out and burning down.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "sphere",
    "radius": 0.2
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 16
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.18,
    0.32
  ],
  "speed": [
    2.5,
    4.5
  ],
  "size": [
    0.32,
    0.5
  ],
  "sizeOverLife": [
    [
      0,
      0.4
    ],
    [
      0.2,
      1
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    -8,
    8
  ],
  "align": {
    "kind": "velocity",
    "stretch": 0
  },
  "angularVelocity": [
    -10,
    10
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/flame")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.7,
        1.509,
        0.432,
        1
      ]
    ],
    [
      1,
      [
        1.7,
        0.432,
        0.023,
        1
      ]
    ]
  ]
});
