import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Green hearts rising round a healed figure.
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "cone",
    "angleDegrees": 4,
    "radius": 0.4
  },
  "spread": 0,
  "rate": 14,
  "duration": 1,
  "looping": false,
  "lifetime": [
    0.7,
    1
  ],
  "speed": [
    0.8,
    1.3
  ],
  "size": [
    0.1,
    0.14
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
    0.3,
    0
  ],
  "drag": 1,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/heartFlat")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.344,
        0.693,
        0.394,
        1
      ]
    ],
    [
      1,
      [
        0.344,
        0.693,
        0.394,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.131,
        0.455,
        0.15,
        1
      ]
    ],
    [
      1,
      [
        0.131,
        0.455,
        0.15,
        1
      ]
    ]
  ]
});
