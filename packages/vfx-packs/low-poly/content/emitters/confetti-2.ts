import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Confetti Pop: one colour of the confetti.
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "cone",
    "angleDegrees": 30,
    "radius": 0.1
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 12
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
    6.5
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
    "mesh": ref("base/vfx/low-poly/models/ribbon")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.042,
        0.238,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.042,
        0.238,
        0.871,
        1
      ]
    ]
  ]
});
