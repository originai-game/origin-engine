import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A few chips.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.08
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 7
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.7,
    1.2
  ],
  "speed": [
    2.5,
    4.5
  ],
  "size": [
    0.06,
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
    -420,
    420
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/card")
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
        0.719,
        0.297,
        0.068,
        1
      ]
    ],
    [
      1,
      [
        0.719,
        0.297,
        0.068,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0.35,
    "friction": 0.5,
    "against": "depth"
  }
});
