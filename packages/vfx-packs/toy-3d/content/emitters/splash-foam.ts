import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Foam puffs on the surface.
export default file.particles({
  "maxParticles": 12,
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
      "count": 8
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4,
    0.7
  ],
  "speed": [
    1.2,
    1.8
  ],
  "size": [
    0.14,
    0.22
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
    0,
    0
  ],
  "drag": 5,
  "wind": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.791,
        0.922,
        1,
        1
      ]
    ],
    [
      1,
      [
        0.791,
        0.922,
        1,
        1
      ]
    ]
  ]
});
