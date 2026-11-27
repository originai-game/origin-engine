import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars rising off a magic circle.
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "cone",
    "angleDegrees": 2,
    "radius": 0.9
  },
  "spread": 0,
  "rate": 22,
  "duration": 2.6,
  "looping": false,
  "lifetime": [
    0.8,
    1.2
  ],
  "speed": [
    0.6,
    1
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
  "drag": 0.6,
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
