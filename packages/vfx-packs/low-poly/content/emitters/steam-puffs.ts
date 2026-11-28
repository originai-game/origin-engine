import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs of steam hissing out of a vent.
export default file.particles({
  "maxParticles": 34,
  "shape": {
    "kind": "cone",
    "angleDegrees": 8,
    "radius": 0.04
  },
  "spread": 0,
  "rate": 24,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    2.2,
    3
  ],
  "size": [
    0.12,
    0.2
  ],
  "sizeOverLife": [
    [
      0,
      0.3
    ],
    [
      0.15,
      0.8
    ],
    [
      0.7,
      1.4
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
    0.5,
    0
  ],
  "drag": 2,
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
  ]
});
