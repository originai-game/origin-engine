import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Rock lumps.
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "sphere",
    "radius": 0.25
  },
  "spread": 70,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 9
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.9,
    1.2
  ],
  "speed": [
    2,
    3.2
  ],
  "size": [
    0.14,
    0.2
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
    -240,
    240
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/block")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.275,
        0.254,
        0.235,
        1
      ]
    ],
    [
      1,
      [
        0.275,
        0.254,
        0.235,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.485,
        0.451,
        0.407,
        1
      ]
    ],
    [
      1,
      [
        0.485,
        0.451,
        0.407,
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
