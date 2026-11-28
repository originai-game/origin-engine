import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A small blast's puffs.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
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
    0.6,
    1
  ],
  "speed": [
    1.6,
    3
  ],
  "size": [
    0.26,
    0.42
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
  "drag": 3.5,
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
        1,
        0.254,
        0.014,
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
