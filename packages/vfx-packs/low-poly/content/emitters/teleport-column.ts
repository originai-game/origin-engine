import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A column of stars lifting the figure away.
export default file.particles({
  "maxParticles": 61,
  "shape": {
    "kind": "cone",
    "angleDegrees": 2,
    "radius": 0.4
  },
  "spread": 0,
  "rate": 60,
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.4,
    0.6
  ],
  "speed": [
    2.5,
    3.5
  ],
  "size": [
    0.06,
    0.1
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.8,
      1
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
    -360,
    360
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 0.5,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.393,
        0.866,
        1.5,
        1
      ]
    ],
    [
      1,
      [
        0.393,
        0.866,
        1.5,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1.447,
        1.395,
        1.307,
        1
      ]
    ],
    [
      1,
      [
        1.447,
        1.395,
        1.307,
        1
      ]
    ]
  ]
});
