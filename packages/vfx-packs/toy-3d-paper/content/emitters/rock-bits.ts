import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Gravel.
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "sphere",
    "radius": 0.25
  },
  "spread": 80,
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
    0.7,
    1.2
  ],
  "speed": [
    2.4,
    4
  ],
  "size": [
    0.05,
    0.08
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
        0.483,
        0.462,
        0.431,
        1
      ]
    ],
    [
      1,
      [
        0.483,
        0.462,
        0.431,
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
