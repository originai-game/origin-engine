import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "blend": "additive",
  "color": [
    1.399999976158142,
    1.399999976158142,
    1.399999976158142,
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
      0.20000000298023224,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.75,
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
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 2,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.699999988079071,
    1.100000023841858
  ],
  "maxParticles": 12,
  "rate": 7,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2199999988079071,
      0.550000011920929,
      0.18000000715255737
    ]
  },
  "sheet": {
    "blend": true,
    "columns": 8,
    "frames": 32,
    "mode": {
      "kind": "fps",
      "fps": 20
    },
    "rows": 4
  },
  "size": [
    0.5,
    0.800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.800000011920929
    ],
    [
      0.5,
      1
    ],
    [
      1,
      0.8999999761581421
    ]
  ],
  "softDistance": 0.10000000149011612,
  "space": "local",
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/flames")
});
