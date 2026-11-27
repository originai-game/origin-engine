import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Magic stars thrown from a spell's hit.
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 14
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.45,
    0.7
  ],
  "speed": [
    2,
    3.5
  ],
  "size": [
    0.08,
    0.14
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
    -2,
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
        0.539,
        0.369,
        0.888,
        1
      ]
    ],
    [
      1,
      [
        0.539,
        0.369,
        0.888,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.852,
        0.263,
        0.433,
        1
      ]
    ],
    [
      1,
      [
        0.852,
        0.263,
        0.433,
        1
      ]
    ]
  ]
});
