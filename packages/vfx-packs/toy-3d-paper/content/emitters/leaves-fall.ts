import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Leaves fluttering down.
export default file.particles({
  "maxParticles": 74,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2,
      0.05,
      2
    ]
  },
  "spread": 0,
  "rate": 10,
  "duration": 3,
  "looping": true,
  "lifetime": [
    3.6,
    4.4
  ],
  "speed": [
    0,
    0.2
  ],
  "size": [
    0.15,
    0.2
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.08,
      1
    ],
    [
      0.92,
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
    -140,
    140
  ],
  "gravity": [
    0,
    -1.6,
    0
  ],
  "drag": 1.2,
  "noise": {
    "strength": 0.8,
    "frequency": 0.8,
    "scroll": 0.3
  },
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
        0.192,
        0.43,
        0.089,
        1
      ]
    ],
    [
      1,
      [
        0.192,
        0.43,
        0.089,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.645,
        0.34,
        0.081,
        1
      ]
    ],
    [
      1,
      [
        0.645,
        0.34,
        0.081,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0,
    "friction": 0.9,
    "against": "depth"
  }
});
