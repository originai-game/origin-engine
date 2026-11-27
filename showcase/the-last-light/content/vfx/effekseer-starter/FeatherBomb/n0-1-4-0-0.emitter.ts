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
  "blend": "alpha",
  "bursts": [
    {
      "count": 26,
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
        1,
        1,
        0
      ]
    ],
    [
      0.19349999725818634,
      [
        1,
        1,
        1,
        0.5019999742507935
      ]
    ],
    [
      0.32260000705718994,
      [
        1,
        1,
        1,
        0.5019999742507935
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
  "duration": 0.0333000011742115,
  "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-4-0-0.effect"),
  "lifetime": [
    0.2167000025510788,
    0.8166999816894531
  ],
  "looping": false,
  "maxParticles": 130,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.4249999523162842,
      1.4249999523162842,
      1.4249999523162842
    ]
  },
  "size": [
    3.5625,
    3.5625
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
    4.275000095367432,
    12.824999809265137
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Splash")
});
