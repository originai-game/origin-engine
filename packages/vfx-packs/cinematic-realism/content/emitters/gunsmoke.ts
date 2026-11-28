import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -40,
    40
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1,
    0.9800000190734863,
    0.949999988079071,
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
        0.3199999928474426
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
    0.15000000596046448,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.2999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 6,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 18,
    "radius": 0.029999999329447746
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
    0.1599999964237213,
    0.25999999046325684
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.5999999046325684
    ]
  ],
  "softDistance": 0.10000000149011612,
  "speed": [
    0.6000000238418579,
    1.399999976158142
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
