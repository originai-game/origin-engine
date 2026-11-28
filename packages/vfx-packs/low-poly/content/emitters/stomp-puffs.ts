import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Dust puffs squeezed out flat along the ground.
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "cone",
    "angleDegrees": 88,
    "radius": 0.15
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
    0.4,
    0.7
  ],
  "speed": [
    2.2,
    3.2
  ],
  "size": [
    0.2,
    0.32
  ],
  "sizeOverLife": [
    [
      0,
      0.25
    ],
    [
      0.12,
      1
    ],
    [
      0.7,
      0.9
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
    -40,
    40
  ],
  "gravity": [
    0,
    0.2,
    0
  ],
  "drag": 5,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/puff")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.694,
        0.485,
        0.262,
        1
      ]
    ],
    [
      1,
      [
        0.694,
        0.485,
        0.262,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.791,
        0.624,
        0.342,
        1
      ]
    ],
    [
      1,
      [
        0.791,
        0.624,
        0.342,
        1
      ]
    ]
  ]
});
