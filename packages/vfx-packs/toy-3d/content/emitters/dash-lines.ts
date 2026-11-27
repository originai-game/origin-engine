import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Speed streaks behind a dash.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3,
      0.35,
      0.05
    ]
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 8
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.15,
    0.25
  ],
  "speed": [
    5,
    7
  ],
  "size": [
    0.06,
    0.08
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
  "heightOverLife": [
    [
      0,
      4
    ],
    [
      1,
      4
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
    0,
    0
  ],
  "drag": 0,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/ribbon")
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
