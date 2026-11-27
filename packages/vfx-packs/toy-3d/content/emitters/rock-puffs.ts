import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A grey puff of stone dust.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "sphere",
    "radius": 0.12
  },
  "spread": 180,
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
    0.5,
    0.8
  ],
  "speed": [
    1,
    1.8
  ],
  "size": [
    0.24,
    0.36
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
  "drag": 3.5,
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
