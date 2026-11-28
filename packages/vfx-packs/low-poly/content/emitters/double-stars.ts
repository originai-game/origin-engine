import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A few stars.
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "cone",
    "angleDegrees": 70,
    "radius": 0.1
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 4
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.3,
    0.45
  ],
  "speed": [
    1.2,
    1.8
  ],
  "size": [
    0.06,
    0.09
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
    -2,
    0
  ],
  "drag": 4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/star")
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
  ]
});
