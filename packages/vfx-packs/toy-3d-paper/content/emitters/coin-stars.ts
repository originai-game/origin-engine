import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Sparkle stars of a coin collected.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 7
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.3,
    0.5
  ],
  "speed": [
    1.4,
    2.4
  ],
  "size": [
    0.06,
    0.1
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
    -1,
    0
  ],
  "drag": 4,
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
        0.844,
        0.757,
        0.344,
        1
      ]
    ],
    [
      1,
      [
        0.844,
        0.757,
        0.344,
        1
      ]
    ]
  ]
});
