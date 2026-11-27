import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stars bounced up from the stomp.
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "cone",
    "angleDegrees": 30,
    "radius": 0.05
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 5
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.35,
    0.6
  ],
  "speed": [
    2,
    3
  ],
  "size": [
    0.1,
    0.16
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
    -360,
    360
  ],
  "gravity": [
    0,
    -6,
    0
  ],
  "drag": 4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d-paper/models/starFlat")
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
  ]
});
