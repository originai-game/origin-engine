import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars swirling into a portal.
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "sphere",
    "radius": 0.75,
    "surface": true
  },
  "spread": 0,
  "rate": 24,
  "duration": 2,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.05,
    0.08
  ],
  "sizeOverLife": [
    [
      0,
      0
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
  "drag": 0,
  "attractors": [
    {
      "position": [
        0,
        0,
        0
      ],
      "strength": 6,
      "radius": 1
    }
  ],
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.85,
        0.788,
        0.698,
        1
      ]
    ],
    [
      1,
      [
        0.85,
        0.788,
        0.698,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.503,
        0.344,
        0.829,
        1
      ]
    ],
    [
      1,
      [
        0.503,
        0.344,
        0.829,
        1
      ]
    ]
  ]
});
