import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs popping out of a kart's exhaust.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "cone",
    "angleDegrees": 10,
    "radius": 0.02
  },
  "spread": 0,
  "rate": 18,
  "duration": 0.6,
  "looping": true,
  "lifetime": [
    0.4,
    0.65
  ],
  "speed": [
    0.8,
    1.2
  ],
  "size": [
    0.08,
    0.13
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
    0.6,
    0
  ],
  "drag": 3,
  "wind": 0.6,
  "inheritVelocity": 0.2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.807,
        0.776,
        0.738,
        1
      ]
    ],
    [
      1,
      [
        0.807,
        0.776,
        0.738,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.397,
        0.386,
        0.371,
        1
      ]
    ],
    [
      1,
      [
        0.397,
        0.386,
        0.371,
        1
      ]
    ]
  ]
});
