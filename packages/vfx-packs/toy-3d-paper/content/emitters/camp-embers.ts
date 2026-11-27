import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Embers drifting up.
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
  },
  "spread": 20,
  "rate": 6,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.8,
    1.2
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.025,
    0.04
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
    0,
    0
  ],
  "gravity": [
    0,
    0.8,
    0
  ],
  "drag": 0.5,
  "noise": {
    "strength": 0.6,
    "frequency": 2,
    "scroll": 0.4
  },
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/coinFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.007,
        0.416,
        0.095,
        1
      ]
    ],
    [
      1,
      [
        1.007,
        0.416,
        0.095,
        1
      ]
    ]
  ]
});
