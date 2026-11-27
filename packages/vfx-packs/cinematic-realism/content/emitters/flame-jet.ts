import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        2.5999999046325684,
        2.200000047683716,
        1.600000023841858,
        0.8999999761581421
      ]
    ],
    [
      0.20000000298023224,
      [
        2.200000047683716,
        1.5,
        0.699999988079071,
        0.8999999761581421
      ]
    ],
    [
      0.6000000238418579,
      [
        1.600000023841858,
        0.699999988079071,
        0.20000000298023224,
        0.6000000238418579
      ]
    ],
    [
      1,
      [
        0.4000000059604645,
        0.07999999821186066,
        0.019999999552965164,
        0
      ]
    ]
  ],
  "drag": 2.200000047683716,
  "duration": 2,
  "gravity": [
    0,
    1.600000023841858,
    0
  ],
  "lifetime": [
    0.550000011920929,
    0.8500000238418579
  ],
  "maxParticles": 160,
  "rate": 150,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 5,
    "radius": 0.029999999329447746
  },
  "sheet": {
    "blend": false,
    "columns": 8,
    "frames": 32,
    "mode": {
      "kind": "random"
    },
    "rows": 4
  },
  "size": [
    0.20000000298023224,
    0.2800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      0.30000001192092896,
      2
    ],
    [
      1,
      5
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    9,
    13
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/flames")
});
