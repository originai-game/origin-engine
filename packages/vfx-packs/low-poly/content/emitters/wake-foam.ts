import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Foam puffs churned behind a toy boat.
export default file.particles({
  "maxParticles": 46,
  "shape": {
    "kind": "cone",
    "angleDegrees": 80,
    "radius": 0.15
  },
  "spread": 0,
  "rate": 30,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.5,
    1
  ],
  "size": [
    0.12,
    0.2
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
    0,
    0
  ],
  "drag": 2,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/puff")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.791,
        0.922,
        1,
        1
      ]
    ],
    [
      1,
      [
        0.791,
        0.922,
        1,
        1
      ]
    ]
  ]
});
