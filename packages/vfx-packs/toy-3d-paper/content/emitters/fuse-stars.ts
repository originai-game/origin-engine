import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Sparks spitting from a lit fuse.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "sphere",
    "radius": 0.02
  },
  "spread": 180,
  "rate": 40,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.15,
    0.3
  ],
  "speed": [
    1.2,
    2
  ],
  "size": [
    0.04,
    0.07
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
    -3,
    0
  ],
  "drag": 4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.181,
        1.06,
        0.481,
        1
      ]
    ],
    [
      1,
      [
        1.181,
        1.06,
        0.481,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
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
