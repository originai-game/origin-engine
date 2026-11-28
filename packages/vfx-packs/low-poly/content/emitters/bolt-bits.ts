import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little stars shed by a magic bolt.
export default file.particles({
  "maxParticles": 24,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 180,
  "rate": 30,
  "duration": 0.6,
  "looping": true,
  "lifetime": [
    0.25,
    0.45
  ],
  "speed": [
    0.2,
    0.6
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
    -1,
    0
  ],
  "drag": 4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.739,
        0.42,
        1.6,
        1
      ]
    ],
    [
      1,
      [
        0.739,
        0.42,
        1.6,
        1
      ]
    ]
  ]
});
