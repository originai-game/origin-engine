import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      0,
      1
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.10109999775886536,
        1,
        1,
        0.3921999931335449
      ]
    ],
    [
      1,
      [
        0.10109999775886536,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "heightOverLife": [
    [
      0,
      7.5
    ],
    [
      1,
      7.5
    ]
  ],
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 13,
  "rate": 0,
  "rotation": [
    90,
    90
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    2,
    2
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    105,
    105
  ],
  "speedOverLife": [
    [
      0,
      2.126699924468994
    ],
    [
      0.25,
      1.2146999835968018
    ],
    [
      0.375,
      0.9107000231742859
    ],
    [
      0.5,
      0.7139000296592712
    ],
    [
      0.625,
      0.6244999766349792
    ],
    [
      0.75,
      0.6424000263214111
    ],
    [
      1,
      0.991100013256073
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Line01")
});
