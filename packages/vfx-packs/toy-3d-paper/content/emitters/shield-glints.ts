import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Glints twinkling on the shield.
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "sphere",
    "radius": 0.56,
    "surface": true
  },
  "spread": 0,
  "rate": 6,
  "duration": 2,
  "looping": true,
  "lifetime": [
    0.4,
    0.6
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.06,
    0.09
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.3,
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
    -360,
    360
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 0,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.85,
        0.788,
        0.698,
        1
      ]
    ],
    [
      1,
      [
        0.85,
        0.788,
        0.698,
        1
      ]
    ]
  ]
});
