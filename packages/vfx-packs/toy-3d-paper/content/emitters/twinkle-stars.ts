import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars blinking on something shiny.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.3
  },
  "spread": 180,
  "rate": 7,
  "duration": 1.6,
  "looping": true,
  "lifetime": [
    0.5,
    0.7
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.12,
    0.18
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.4,
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
    -90,
    90
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
        0.972,
        0.901,
        0.797,
        1
      ]
    ],
    [
      1,
      [
        0.972,
        0.901,
        0.797,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.945,
        0.848,
        0.385,
        1
      ]
    ],
    [
      1,
      [
        0.945,
        0.848,
        0.385,
        1
      ]
    ]
  ]
});
