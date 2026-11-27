import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Coins fountaining from a reward.
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "cone",
    "angleDegrees": 40,
    "radius": 0.1
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 12
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.9,
    1.3
  ],
  "speed": [
    3,
    4.5
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
    -700,
    700
  ],
  "gravity": [
    0,
    -8,
    0
  ],
  "drag": 0.5,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/coinFlat")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.662,
        0.393,
        0.059,
        1
      ]
    ],
    [
      1,
      [
        0.662,
        0.393,
        0.059,
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
