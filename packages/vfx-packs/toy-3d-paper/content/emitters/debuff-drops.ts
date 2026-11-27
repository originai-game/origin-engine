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
    "mesh": ref("base/vfx/toy-3d-paper/models/dropFlat")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.311,
        0.178,
        0.639,
        1
      ]
    ],
    [
      1,
      [
        0.311,
        0.178,
        0.639,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.121,
        0.112,
        0.106,
        1
      ]
    ],
    [
      1,
      [
        0.121,
        0.112,
        0.106,
        1
      ]
    ]
  ]
});
