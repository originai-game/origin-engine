import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A dull cloud over its head.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 6
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.3,
    0.6
  ],
  "size": [
    0.16,
    0.26
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
    -40,
    40
  ],
  "gravity": [
    0,
    -0.3,
    0
  ],
  "drag": 3.5,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/puff")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.254,
        0.102,
        0.791,
        1
      ]
    ],
    [
      1,
      [
        0.254,
        0.102,
        0.791,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.105,
        0.093,
        0.087,
        1
      ]
    ],
    [
      1,
      [
        0.105,
        0.093,
        0.087,
        1
      ]
    ]
  ]
});
