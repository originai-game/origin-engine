import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Sparks popping off a shocked figure.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "sphere",
    "radius": 0.3
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 4
    },
    {
      "time": 0.15,
      "count": 4
    },
    {
      "time": 0.32,
      "count": 4
    },
    {
      "time": 0.46,
      "count": 4
    }
  ],
  "duration": 0.6,
  "looping": true,
  "lifetime": [
    0.15,
    0.3
  ],
  "speed": [
    1.5,
    2.5
  ],
  "size": [
    0.09,
    0.13
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
        1.063,
        0.954,
        0.433,
        1
      ]
    ],
    [
      1,
      [
        1.063,
        0.954,
        0.433,
        1
      ]
    ]
  ]
});
