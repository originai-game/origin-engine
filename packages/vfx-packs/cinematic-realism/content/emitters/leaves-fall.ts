import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -120,
    120
  ],
  "blend": "alpha",
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 1
  },
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.05000000074505806,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.8999999761581421,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 0.8999999761581421,
  "duration": 2,
  "gravity": [
    0,
    -0.6000000238418579,
    0
  ],
  "lifetime": [
    5,
    7
  ],
  "lit": true,
  "maxParticles": 32,
  "noise": {
    "frequency": 0.5,
    "scroll": 0.30000001192092896,
    "strength": 0.6000000238418579
  },
  "rate": 3.5,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      2,
      0.20000000298023224,
      2
    ]
  },
  "sheet": {
    "blend": false,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "random"
    },
    "rows": 2
  },
  "size": [
    0.07999999821186066,
    0.12999999523162842
  ],
  "speed": [
    0.10000000149011612,
    0.4000000059604645
  ],
  "spread": 180,
  "texture": ref("base/vfx/cinematic-realism/textures/leaves"),
  "wind": 1
});
