import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars rising round the figure.
export default file.particles({
  "maxParticles": 46,
  "shape": {
    "kind": "cone",
    "angleDegrees": 5,
    "radius": 0.5
  },
  "spread": 0,
  "rate": 30,
  "duration": 0.8,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    1.2,
    2
  ],
  "size": [
    0.07,
    0.11
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
  "drag": 1.5,
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
