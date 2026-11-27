import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Ice crystals growing on a frozen figure.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.45,
      0.25
    ]
  },
  "spread": 180,
  "rate": 10,
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
    0.08,
    0.14
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
    0,
    0
  ],
  "drag": 2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/gemFlat")
  },
  "lit": true,
  "roughness": 0.85,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.468,
        0.739,
        0.88,
        1
      ]
    ],
    [
      1,
      [
        0.468,
        0.739,
        0.88,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
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
