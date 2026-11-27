import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 6,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.41999998688697815,
    0.029999999329447746,
    0.02500000037252903,
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
      0.05999999865889549,
      [
        1,
        1,
        1,
        0.75
      ]
    ],
    [
      0.4000000059604645,
      [
        0.699999988079071,
        0.699999988079071,
        0.699999988079071,
        0.4000000059604645
      ]
    ],
    [
      1,
      [
        0.5,
        0.5,
        0.5,
        0
      ]
    ]
  ],
  "drag": 6,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -0.6000000238418579,
    0
  ],
  "lifetime": [
    0.3499999940395355,
    0.699999988079071
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 30,
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
    0.11999999731779099,
    0.20000000298023224
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      0.30000001192092896,
      1.600000023841858
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.07999999821186066,
  "speed": [
    1,
    2.5999999046325684
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
