import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Planks of a broken toy crate.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.25,
      0.25
    ]
  },
  "spread": 70,
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
    0.9,
    1.2
  ],
  "speed": [
    2,
    3.4
  ],
  "size": [
    0.12,
    0.16
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
      0.3
    ],
    [
      1,
      0.3
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "angularVelocity": [
    -360,
    360
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/card")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.465,
        0.284,
        0.145,
        1
      ]
    ],
    [
      1,
      [
        0.465,
        0.284,
        0.145,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.248,
        0.129,
        0.06,
        1
      ]
    ],
    [
      1,
      [
        0.248,
        0.129,
        0.06,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0.35,
    "friction": 0.5,
    "against": "depth"
  }
});
