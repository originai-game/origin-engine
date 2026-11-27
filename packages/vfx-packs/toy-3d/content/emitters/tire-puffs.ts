import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs of a spinning tyre.
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.08
  },
  "spread": 0,
  "rate": 24,
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
    0.16,
    0.26
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
    0.4,
    0
  ],
  "drag": 2,
  "wind": 0.6,
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
        0.965,
        0.93,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.965,
        0.93,
        0.871,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
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
  ]
});
