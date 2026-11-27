import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Sparkles at the laser's mouth.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 10
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.2,
    0.35
  ],
  "speed": [
    1,
    2
  ],
  "size": [
    0.035,
    0.06
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
    0,
    0
  ],
  "gravity": [
    0,
    -3,
    0
  ],
  "drag": 2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        2,
        0.665,
        1.054,
        1
      ]
    ],
    [
      1,
      [
        2,
        0.665,
        1.054,
        1
      ]
    ]
  ]
});
