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
    0.949999988079071,
    0.949999988079071,
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
  "maxParticles": 16,
  "rate": 9,
  "shape": {
    "kind": "sphere",
    "radius": 0.6000000238418579,
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
    1.7999999523162842,
    2.5999999046325684
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
  "softDistance": 0.5,
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/flames")
});
