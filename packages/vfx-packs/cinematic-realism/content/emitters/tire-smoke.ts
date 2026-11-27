import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -25,
    25
  ],
  "blend": "alpha",
  "color": [
    1.7000000476837158,
    1.7000000476837158,
    1.75,
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
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.44999998807907104
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        1,
        0.25999999046325684
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
  "drag": 1.7999999523162842,
  "duration": 2,
  "gravity": [
    0,
    0.3499999940395355,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    2.200000047683716
  ],
  "lit": true,
  "maxParticles": 120,
  "noise": {
    "frequency": 0.699999988079071,
    "scroll": 0.30000001192092896,
    "strength": 0.30000001192092896
  },
  "rate": 50,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 70,
    "radius": 0.10000000149011612
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
    0.30000001192092896,
    0.5
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      0.30000001192092896,
      1
    ],
    [
      1,
      2.5999999046325684
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.5,
    1.600000023841858
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
