import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars pulled in to a charging spell.
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "sphere",
    "radius": 0.6,
    "surface": true
  },
  "spread": 0,
  "rate": 36,
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.45,
    0.6
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.06,
    0.09
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
  "drag": 0,
  "attractors": [
    {
      "position": [
        0,
        0,
        0
      ],
      "strength": 14,
      "radius": 1
    }
  ],
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.42,
        0.924,
        1.6,
        1
      ]
    ],
    [
      1,
      [
        0.42,
        0.924,
        1.6,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1.543,
        1.488,
        1.394,
        1
      ]
    ],
    [
      1,
      [
        1.543,
        1.488,
        1.394,
        1
      ]
    ]
  ]
});
