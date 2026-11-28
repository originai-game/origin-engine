import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Dust kicked up by a wheel.
export default file.particles({
  "maxParticles": 28,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.08
  },
  "spread": 0,
  "rate": 20,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.14,
    0.22
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
    0.3,
    0
  ],
  "drag": 2.5,
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
        0.694,
        0.485,
        0.262,
        1
      ]
    ],
    [
      1,
      [
        0.694,
        0.485,
        0.262,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.791,
        0.624,
        0.342,
        1
      ]
    ],
    [
      1,
      [
        0.791,
        0.624,
        0.342,
        1
      ]
    ]
  ]
});
