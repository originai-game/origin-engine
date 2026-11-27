import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Smaller bits.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.25,
      0.25
    ]
  },
  "spread": 80,
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
    1.2
  ],
  "speed": [
    2.4,
    4
  ],
  "size": [
    0.08,
    0.1
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
    "mesh": ref("base/vfx/toy-3d/models/block")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.644,
        0.042,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.644,
        0.042,
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
