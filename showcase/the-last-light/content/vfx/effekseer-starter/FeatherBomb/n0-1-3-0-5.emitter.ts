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
  "blend": "alpha",
  "bursts": [
    {
      "count": 52,
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
      0.14059999585151672,
      [
        1,
        1,
        1,
        0.15919999778270721
      ]
    ],
    [
      0.2856999933719635,
      [
        1,
        1,
        1,
        0.2689000070095062
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
  "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-5.effect"),
  "lifetime": [
    0.05000000074505806,
    0.6499999761581421
  ],
  "looping": false,
  "maxParticles": 130,
  "rate": 0,
  "rotation": [
    -30,
    30
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
    1.7812000513076782,
    1.7812000513076782
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
