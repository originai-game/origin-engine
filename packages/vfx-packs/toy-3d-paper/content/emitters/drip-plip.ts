import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). The little crown of drops where a drip lands.
export default file.particles({
  "maxParticles": 2,
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.01
  },
  "rate": 0,
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.2,
    0.35
  ],
  "speed": [
    0.8,
    1.4
  ],
  "size": [
    0.025,
    0.04
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
