import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Little hearts floating up.
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "sphere",
    "radius": 0.15
  },
  "spread": 180,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 6
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.08,
    0.12
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
    1.2,
    0
  ],
  "drag": 2,
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
        0.811,
        0.25,
        0.412,
        1
      ]
    ],
    [
      1,
      [
        0.811,
        0.25,
        0.412,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.866,
        0.415,
        0.545,
        1
      ]
    ],
    [
      1,
      [
        0.866,
        0.415,
        0.545,
        1
      ]
    ]
  ]
});
