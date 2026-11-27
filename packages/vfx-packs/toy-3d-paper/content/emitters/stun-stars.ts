import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars blinking on the ring round a dizzy head.
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "cone",
    "angleDegrees": 90,
    "radius": 0.24
  },
  "spread": 0,
  "rate": 7,
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.35,
    0.5
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.1,
    0.13
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.25,
      1
    ],
    [
      0.75,
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
    -200,
    200
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
