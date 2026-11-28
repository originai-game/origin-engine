import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars where it lands.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
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
    0.25,
    0.4
  ],
  "speed": [
    1.5,
    2.5
  ],
  "size": [
    0.1,
    0.16
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
