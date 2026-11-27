import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 14,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.6200000047683716,
    0.5699999928474426,
    0.5,
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
        0.8500000238418579
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.44999998807907104
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
  "drag": 2.5999999046325684,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.07999999821186066,
    0
  ],
  "lifetime": [
    2,
    3
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
    "angleDegrees": 88,
    "radius": 0.6000000238418579
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
    0.699999988079071,
    1.100000023841858
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      0.4000000059604645,
      1.2000000476837158
    ],
    [
      1,
      1.7999999523162842
    ]
  ],
  "softDistance": 0.4000000059604645,
  "speed": [
    3,
    5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
