import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Puffs ploughed up by a skid.
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "cone",
    "angleDegrees": 70,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 22,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.35,
    0.55
  ],
  "speed": [
    0.6,
    1.2
  ],
  "size": [
    0.12,
    0.18
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
    0.3,
    0
  ],
  "drag": 3,
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
