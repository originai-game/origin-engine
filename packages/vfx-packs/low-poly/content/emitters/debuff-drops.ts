import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Dark drops falling round a weakened figure.
export default file.particles({
  "maxParticles": 23,
  "shape": {
    "kind": "cone",
    "angleDegrees": 4,
    "radius": 0.35
  },
  "spread": 0,
  "rate": 18,
  "duration": 0.8,
  "looping": false,
  "lifetime": [
    0.5,
    0.7
  ],
  "speed": [
    0.6,
    1
  ],
  "size": [
    0.08,
    0.12
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
    -3,
    0
  ],
  "drag": 0.5,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/drop")
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
