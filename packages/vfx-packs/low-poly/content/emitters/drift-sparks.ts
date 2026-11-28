import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Blue sparks thrown off a drifting wheel: a mini-turbo charging.
export default file.particles({
  "maxParticles": 23,
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 40,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.18,
    0.32
  ],
  "speed": [
    1.6,
    2.8
  ],
  "size": [
    0.05,
    0.08
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
    -6,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.525,
        1.155,
        2,
        1
      ]
    ],
    [
      1,
      [
        0.525,
        1.155,
        2,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1.929,
        1.86,
        1.743,
        1
      ]
    ],
    [
      1,
      [
        1.929,
        1.86,
        1.743,
        1
      ]
    ]
  ]
});
