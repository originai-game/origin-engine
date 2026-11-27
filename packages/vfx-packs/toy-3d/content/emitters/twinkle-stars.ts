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
    "mesh": ref("base/vfx/toy-3d/models/star")
  },
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1.543,
        1.488,
        1.394,
        1
      ]
    ],
    [
      1,
      [
        1.543,
        1.488,
        1.394,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        1.6,
        1.421,
        0.407,
        1
      ]
    ],
    [
      1,
      [
        1.6,
        1.421,
        0.407,
        1
      ]
    ]
  ]
});
