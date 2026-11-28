import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little hearts floating up.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 6
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.08,
    0.12
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
    -60,
    60
  ],
  "gravity": [
    0,
    1.2,
    0
  ],
  "drag": 2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/heart")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.159,
        0.392,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.159,
        0.392,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1,
        0.332,
        0.527,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.332,
        0.527,
        1
      ]
    ]
  ]
});
