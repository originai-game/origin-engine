import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A toy fountain's jet of drops.
export default file.particles({
  "maxParticles": 70,
  "shape": {
    "kind": "cone",
    "angleDegrees": 5,
    "radius": 0.03
  },
  "rate": 46,
  "duration": 1,
  "looping": true,
  "lifetime": [
    0.75,
    0.9
  ],
  "speed": [
    3.6,
    4.2
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
    -60,
    60
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/dropFlat")
  },
  "lit": true,
  "roughness": 0.85,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.144,
        0.406,
        0.678,
        1
      ]
    ],
    [
      1,
      [
        0.144,
        0.406,
        0.678,
        1
      ]
    ]
  ]
});
