import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A ring of little stars bursting out.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "sphere",
    "radius": 0.05,
    "surface": true
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 10
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4,
    0.6
  ],
  "speed": [
    2,
    2.4
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
    -360,
    360
  ],
  "gravity": [
    0,
    0,
    0
  ],
  "drag": 3,
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
