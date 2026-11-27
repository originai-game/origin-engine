import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A ring of puffs flattened out by a landing.
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
    0.45,
    0.7
  ],
  "speed": [
    2,
    2.8
  ],
  "size": [
    0.18,
    0.28
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
  "drag": 6,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
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
