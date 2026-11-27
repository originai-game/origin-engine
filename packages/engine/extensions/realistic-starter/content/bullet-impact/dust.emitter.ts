import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -30,
    30
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
    0.8199999928474426,
    0.7699999809265137,
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
      0.07999999821186066,
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
  "drag": 3,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.10000000149011612,
    0
  ],
  "lifetime": [
    0.8999999761581421,
    1.5
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
    0.14000000059604645,
    0.23999999463558197
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0.6000000238418579,
    1.7999999523162842
  ],
  "spread": 10,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
