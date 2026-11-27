import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Flakes drifting off it.
export default file.particles({
  "maxParticles": 18,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3,
      0.1,
      0.3
    ]
  },
  "spread": 180,
  "rate": 8,
  "duration": 1.5,
  "looping": true,
  "lifetime": [
    0.8,
    1.2
  ],
  "speed": [
    0.1,
    0.3
  ],
  "size": [
    0.04,
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
    -360,
    360
  ],
  "gravity": [
    0,
    -0.4,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.868,
        0.804,
        0.712,
        1
      ]
    ],
    [
      1,
      [
        0.868,
        0.804,
        0.712,
        1
      ]
    ]
  ]
});
