import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.8999999761581421,
    0.9399999976158142,
    0.9800000190734863,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.75
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
  "drag": 1,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -7,
    0
  ],
  "lifetime": [
    0.5,
    0.800000011920929
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 12,
    "radius": 0.07999999821186066
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
    0.18000000715255737,
    0.3199999928474426
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.600000023841858
    ]
  ],
  "softDistance": 0.15000000596046448,
  "speed": [
    2,
    4
  ],
  "spread": 6,
  "texture": ref("base/vfx/realistic-starter/textures/smoke")
});
