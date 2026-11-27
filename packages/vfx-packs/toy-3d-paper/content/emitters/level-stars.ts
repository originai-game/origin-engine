import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars rising round a levelled figure.
export default file.particles({
  "maxParticles": 83,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.6
  },
  "spread": 0,
  "rate": 45,
  "duration": 1,
  "looping": false,
  "lifetime": [
    0.8,
    1.1
  ],
  "speed": [
    1.6,
    2.6
  ],
  "size": [
    0.08,
    0.13
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
    0,
    0
  ],
  "drag": 1.2,
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
        0.75,
        0.522,
        0.115,
        1
      ]
    ],
    [
      1,
      [
        0.75,
        0.522,
        0.115,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
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
