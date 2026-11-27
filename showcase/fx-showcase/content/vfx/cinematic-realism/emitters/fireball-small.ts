import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
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
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.30000001192092896,
    0
  ],
  "lifetime": [
    1.600000023841858,
    1.600000023841858
  ],
  "looping": false,
  "maxParticles": 1,
  "rate": 0,
  "shape": {
    "kind": "point"
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
    1.600000023841858,
    1.600000023841858
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/explosion")
});
