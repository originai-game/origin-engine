import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0.20000000298023224
    }
  ],
  "color": [
    0.949999988079071,
    1.0800000429153442,
    1.2000000476837158,
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
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.3499999940395355
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
  "drag": 1.5,
  "duration": 0.30000001192092896,
  "gravity": [
    0,
    0.05000000074505806,
    0
  ],
  "lifetime": [
    2.5,
    4
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 1
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
    2,
    3
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      1.7999999523162842
    ]
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    0.6000000238418579,
    1.600000023841858
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
