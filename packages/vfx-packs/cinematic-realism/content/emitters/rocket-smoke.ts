import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -25,
    25
  ],
  "blend": "alpha",
  "color": [
    0.8199999928474426,
    0.800000011920929,
    0.7799999713897705,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1.600000023841858,
        1.2000000476837158,
        0.800000011920929,
        0
      ]
    ],
    [
      0.03999999910593033,
      [
        1.5,
        1.149999976158142,
        0.800000011920929,
        0.8500000238418579
      ]
    ],
    [
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.699999988079071
      ]
    ],
    [
      0.6000000238418579,
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
  "duration": 2,
  "gravity": [
    0,
    0.11999999731779099,
    0
  ],
  "lifetime": [
    1.7999999523162842,
    2.799999952316284
  ],
  "lit": true,
  "maxParticles": 400,
  "noise": {
    "frequency": 0.800000011920929,
    "scroll": 0.30000001192092896,
    "strength": 0.25
  },
  "rate": 140,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.03999999910593033,
    "surface": false
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
    0.4000000059604645,
    0.6000000238418579
  ],
  "sizeOverLife": [
    [
      0,
      0.699999988079071
    ],
    [
      0.20000000298023224,
      1.2000000476837158
    ],
    [
      1,
      3
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0.10000000149011612,
    0.5
  ],
  "spread": 180,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
