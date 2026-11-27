import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.8500000238418579,
    0.8600000143051147,
    0.8799999952316284,
    1
  ],
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
      0.10000000149011612,
      [
        1,
        1,
        1,
        0.699999988079071
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
  "drag": 3,
  "duration": 1.2000000476837158,
  "gravity": [
    0,
    0.20000000298023224,
    0
  ],
  "lifetime": [
    0.5,
    0.8999999761581421
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 40,
  "rate": 30,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 20,
    "radius": 0.019999999552965164
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    0.10000000149011612,
    0.1599999964237213
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      4
    ]
  ],
  "softDistance": 0.10000000149011612,
  "speed": [
    2,
    4
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
