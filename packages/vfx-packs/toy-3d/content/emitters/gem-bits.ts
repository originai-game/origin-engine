import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Glinting gem chips.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "sphere",
    "radius": 0.05
  },
  "spread": 180,
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
    0.35,
    0.55
  ],
  "speed": [
    1.5,
    2.6
  ],
  "size": [
    0.05,
    0.08
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
    0,
    0
  ],
  "gravity": [
    0,
    -2,
    0
  ],
  "drag": 2,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/gem")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.392,
        0.761,
        1,
        1
      ]
    ],
    [
      1,
      [
        0.392,
        0.761,
        1,
        1
      ]
    ]
  ]
});
