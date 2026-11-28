import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Green bubbles rising off a poisoned figure.
export default file.particles({
  "maxParticles": 23,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.35,
      0.2
    ]
  },
  "spread": 0,
  "rate": 14,
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.4,
    0.8
  ],
  "size": [
    0.07,
    0.13
  ],
  "sizeOverLife": [
    [
      0,
      0.25
    ],
    [
      0.12,
      1
    ],
    [
      0.7,
      0.9
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
    0.6,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/puff")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.238,
        0.638,
        0.042,
        1
      ]
    ],
    [
      1,
      [
        0.238,
        0.638,
        0.042,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.462,
        0.761,
        0.068,
        1
      ]
    ],
    [
      1,
      [
        0.462,
        0.761,
        0.068,
        1
      ]
    ]
  ]
});
