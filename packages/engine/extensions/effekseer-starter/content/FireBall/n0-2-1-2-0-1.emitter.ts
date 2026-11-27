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
    -180,
    180
  ],
  "blend": "additive",
  "bursts": [
    {
      "count": 27,
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
      0.19349999725818634,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.5019999742507935
      ]
    ],
    [
      0.32260000705718994,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.5019999742507935
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
  "effect": ref("base/vfx/effekseer-starter/FireBall/n0-2-1-2-0-1.effect"),
  "lifetime": [
    0.2167000025510788,
    0.8166999816894531
  ],
  "looping": false,
  "maxParticles": 135,
  "rate": 0,
  "rotation": [
    -360,
    360
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
    4.75,
    4.75
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
