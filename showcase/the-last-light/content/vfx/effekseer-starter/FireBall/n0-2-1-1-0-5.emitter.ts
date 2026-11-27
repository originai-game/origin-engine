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
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "additive",
  "bursts": [
    {
      "count": 54,
      "cycles": 2,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0
      ]
    ],
    [
      0.14059999585151672,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.15919999778270721
      ]
    ],
    [
      0.2856999933719635,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.2689000070095062
      ]
    ],
    [
      1,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0
      ]
    ]
  ],
  "duration": 0.0333000011742115,
  "effect": ref("base/vfx/effekseer-starter/FireBall/n0-2-1-1-0-5.effect"),
  "lifetime": [
    0.05000000074505806,
    0.6499999761581421
  ],
  "looping": false,
  "maxParticles": 135,
  "rate": 0,
  "rotation": [
    -30,
    30
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.899999976158142,
      1.899999976158142,
      1.899999976158142
    ]
  },
  "size": [
    2.375,
    2.375
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      0.21879999339580536,
      0.5723000168800354
    ],
    [
      0.4530999958515167,
      0.7268999814987183
    ],
    [
      0.7031000256538391,
      0.8641999959945679
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    5.699999809265137,
    17.100000381469727
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Splash")
});
