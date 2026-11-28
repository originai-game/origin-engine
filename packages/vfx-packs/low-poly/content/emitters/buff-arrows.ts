import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Drops shooting up round a strengthened figure.
export default file.particles({
  "maxParticles": 28,
  "shape": {
    "kind": "cone",
    "angleDegrees": 3,
    "radius": 0.42
  },
  "spread": 0,
  "rate": 22,
  "duration": 0.8,
  "looping": false,
  "lifetime": [
    0.5,
    0.7
  ],
  "speed": [
    1.6,
    2.4
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
    0,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/drop")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.254,
        0.014,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.254,
        0.014,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1,
        0.644,
        0.042,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.644,
        0.042,
        1
      ]
    ]
  ]
});
