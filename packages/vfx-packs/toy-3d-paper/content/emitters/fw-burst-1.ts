import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A burst of coloured stars.
export default file.particles({
  "maxParticles": 40,
  "shape": {
    "kind": "sphere",
    "radius": 0.1,
    "surface": true
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 30
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    1.1,
    1.5
  ],
  "speed": [
    4.5,
    5.5
  ],
  "size": [
    0.12,
    0.18
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
    -1.5,
    0
  ],
  "drag": 1.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
  },
  "blend": "alpha",
  "trail": {
    "lifetime": 0.18,
    "width": 0.4
  },
  "colorOverLife": [
    [
      0,
      [
        0.944,
        0.657,
        0.145,
        1
      ]
    ],
    [
      1,
      [
        0.944,
        0.657,
        0.145,
        1
      ]
    ]
  ]
});
