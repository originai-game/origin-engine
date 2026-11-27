import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.019999999552965164
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    6,
    3,
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
        1
      ]
    ],
    [
      1,
      [
        1,
        0.4000000059604645,
        0.15000000596046448,
        0
      ]
    ]
  ],
  "drag": 8,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "heightOverLife": [
    [
      0,
      2.5
    ],
    [
      1,
      4
    ]
  ],
  "lifetime": [
    0.05000000074505806,
    0.07999999821186066
  ],
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 12,
    "radius": 0.019999999552965164
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
    0.07000000029802322,
    0.11999999731779099
  ],
  "speed": [
    5,
    9
  ],
  "texture": ref("base/vfx/realistic-starter/textures/flame")
});
