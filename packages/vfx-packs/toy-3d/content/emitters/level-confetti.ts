import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A pop of confetti.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "cone",
    "angleDegrees": 20,
    "radius": 0.15
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 16
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    1.6,
    2.4
  ],
  "speed": [
    4,
    6
  ],
  "size": [
    0.12,
    0.17
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
    -540,
    540
  ],
  "gravity": [
    0,
    -4,
    0
  ],
  "drag": 2.6,
  "noise": {
    "strength": 1.2,
    "frequency": 1.5,
    "scroll": 0.5
  },
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/ribbon")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
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
  ],
  "randomColorOverLife": [
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
  ]
});
