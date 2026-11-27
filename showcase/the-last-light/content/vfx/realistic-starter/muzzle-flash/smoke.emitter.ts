import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -40,
    40
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 4,
      "cycles": 1,
      "interval": 0,
      "time": 0.019999999552965164
    }
  ],
  "color": [
    0.75,
    0.7300000190734863,
    0.699999988079071,
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
        0.550000011920929
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
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.25,
    0
  ],
  "lifetime": [
    0.699999988079071,
    1.100000023841858
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
    "angleDegrees": 20,
    "radius": 0.029999999329447746
  },
  "sheet": {
    "blend": true,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 2
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
      2.5999999046325684
    ]
  ],
  "softDistance": 0.15000000596046448,
  "speed": [
    0.800000011920929,
    2
  ],
  "spread": 10,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
