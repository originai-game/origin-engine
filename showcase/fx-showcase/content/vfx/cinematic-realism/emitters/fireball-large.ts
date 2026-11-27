import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 3,
      "cycles": 1,
      "interval": 0,
      "time": 0
    },
    {
      "count": 2,
      "cycles": 1,
      "interval": 0,
      "time": 0.11999999731779099
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.6499999761581421,
        0.6499999761581421,
        0.6499999761581421,
        1
      ]
    ],
    [
      0.20000000298023224,
      [
        0.8500000238418579,
        0.8500000238418579,
        0.8500000238418579,
        1
      ]
    ],
    [
      0.699999988079071,
      [
        1,
        1,
        1,
        0.8999999761581421
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
  "duration": 0.20000000298023224,
  "gravity": [
    0,
    0.5,
    0
  ],
  "lifetime": [
    3.4000000953674316,
    4.199999809265137
  ],
  "looping": false,
  "maxParticles": 5,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 1.2000000476837158,
    "surface": false
  },
  "sheet": {
    "blend": true,
    "columns": 5,
    "frames": 25,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 5
  },
  "size": [
    4.199999809265137,
    5.400000095367432
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    0.20000000298023224,
    0.800000011920929
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/explosion")
});
