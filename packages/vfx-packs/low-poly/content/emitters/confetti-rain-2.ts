import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Confetti fluttering down over an area.
export default file.particles({
  "maxParticles": 75,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2,
      0.05,
      2
    ]
  },
  "rate": 14,
  "duration": 2,
  "looping": true,
  "lifetime": [
    2.6,
    3.2
  ],
  "speed": [
    0,
    0.3
  ],
  "size": [
    0.15,
    0.2
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
    -540,
    540
  ],
  "gravity": [
    0,
    -3,
    0
  ],
  "drag": 1.5,
  "noise": {
    "strength": 1.2,
    "frequency": 1.5,
    "scroll": 0.5
  },
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/ribbon")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.042,
        0.238,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.042,
        0.238,
        0.871,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0.1,
    "friction": 0.9,
    "against": "depth"
  }
});
