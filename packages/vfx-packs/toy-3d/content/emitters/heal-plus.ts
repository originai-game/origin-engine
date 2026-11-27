import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little sparkles among them.
export default file.particles({
  "maxParticles": 26,
  "shape": {
    "kind": "cone",
    "angleDegrees": 4,
    "radius": 0.45
  },
  "spread": 0,
  "rate": 18,
  "duration": 1,
  "looping": false,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    0.8,
    1.4
  ],
  "size": [
    0.05,
    0.08
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
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/star")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.888,
        0.254,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.888,
        0.254,
        1
      ]
    ]
  ]
});
