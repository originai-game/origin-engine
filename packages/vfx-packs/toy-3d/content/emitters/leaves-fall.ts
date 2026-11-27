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
    "mesh": ref("base/vfx/toy-3d/models/ribbon")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.15,
        0.521,
        0.042,
        1
      ]
    ],
    [
      1,
      [
        0.15,
        0.521,
        0.042,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.871,
        0.323,
        0.023,
        1
      ]
    ],
    [
      1,
      [
        0.871,
        0.323,
        0.023,
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
