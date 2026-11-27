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
    1.0499999523162842,
    1.0499999523162842,
    1.0499999523162842,
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
    0.8999999761581421,
    1.2999999523162842
  ],
  "maxParticles": 8,
  "rate": 5,
  "shape": {
    "kind": "sphere",
    "radius": 0.10000000149011612,
    "surface": false
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
    0.699999988079071
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
  "softDistance": 0.15000000596046448,
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/flames")
});
