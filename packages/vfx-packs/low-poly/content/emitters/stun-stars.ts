import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars blinking on the ring round a dizzy head.
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "cone",
    "angleDegrees": 90,
    "radius": 0.24
  },
  "spread": 0,
  "rate": 7,
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.35,
    0.5
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.1,
    0.13
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.25,
      1
    ],
    [
      0.75,
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
    -200,
    200
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 0,
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
  ],
  "randomColorOverLife": [
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
