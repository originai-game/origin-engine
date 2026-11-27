import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -40,
    40
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 2,
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
        0.2199999988079071
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
    0.6000000238418579,
    1
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
    0.14000000059604645,
    0.2199999988079071
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
  "softDistance": 0.10000000149011612,
  "speed": [
    0.6000000238418579,
    1.399999976158142
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
