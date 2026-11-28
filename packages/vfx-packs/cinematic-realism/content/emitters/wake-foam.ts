import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      1,
      0
    ]
  },
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "color": [
    1.7999999523162842,
    1.8600000143051147,
    1.9199999570846558,
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
        0.6000000238418579
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.3400000035762787
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
    0,
    0
  ],
  "lifetime": [
    2,
    3.200000047683716
  ],
  "maxParticles": 110,
  "rate": 36,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0,
      0.15000000596046448
    ]
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
    0.550000011920929,
    0.8999999761581421
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
  "speed": [
    0.20000000298023224,
    0.6000000238418579
  ],
  "spread": 90,
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
