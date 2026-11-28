import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0.07999999821186066
    }
  ],
  "color": [
    0.41999998688697815,
    0.4000000059604645,
    0.3799999952316284,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        0.5,
        0.44999998807907104,
        0.4000000059604645,
        0
      ]
    ],
    [
      0.11999999731779099,
      [
        1,
        1,
        1,
        0.949999988079071
      ]
    ],
    [
      0.6000000238418579,
      [
        1.2999999523162842,
        1.2999999523162842,
        1.2999999523162842,
        0.75
      ]
    ],
    [
      1,
      [
        1.600000023841858,
        1.600000023841858,
        1.600000023841858,
        0
      ]
    ]
  ],
  "drag": 1.2000000476837158,
  "duration": 0.10000000149011612,
  "forceFields": 1,
  "gravity": [
    0,
    0.8999999761581421,
    0
  ],
  "lifetime": [
    2.200000047683716,
    3.299999952316284
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.5,
    "surface": false
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
    1,
    1.5
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.299999952316284
    ]
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    0.6000000238418579,
    2.4000000953674316
  ],
  "spread": 70,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
