import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A slow string of puffs from a chimney.
export default file.particles({
  "maxParticles": 41,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.06
  },
  "spread": 0,
  "rate": 8,
  "duration": 2,
  "looping": true,
  "lifetime": [
    2.4,
    3
  ],
  "speed": [
    0.6,
    0.9
  ],
  "size": [
    0.24,
    0.34
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
      1.2
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
    0.1,
    0
  ],
  "drag": 0.4,
  "wind": 0.8,
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
