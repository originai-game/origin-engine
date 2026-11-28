import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Coins jumping out of an opened chest.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "cone",
    "angleDegrees": 22,
    "radius": 0.12
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
    0.7,
    1
  ],
  "speed": [
    3,
    4.2
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
    -600,
    600
  ],
  "gravity": [
    0,
    -9,
    0
  ],
  "drag": 0.5,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/coin")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.922,
        0.485,
        0.01,
        1
      ]
    ],
    [
      1,
      [
        0.922,
        0.485,
        0.01,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0.4,
    "friction": 0.5,
    "against": "depth"
  }
});
