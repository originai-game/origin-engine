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
    -60,
    60
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 26,
      "cycles": 3,
      "interval": 0.02500000037252903,
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
        1
      ]
    ],
    [
      0.5091000199317932,
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
  "duration": 0.06669999659061432,
  "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-0-0.effect"),
  "gravity": [
    0,
    12.960000038146973,
    -12.960000038146973
  ],
  "lifetime": [
    0.666700005531311,
    1.166700005531311
  ],
  "looping": false,
  "maxParticles": 195,
  "rate": 0,
  "rotation": [
    -15,
    15
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.23999999463558197,
      0.23999999463558197,
      0.23999999463558197
    ]
  },
  "size": [
    0.7200000286102295,
    0.7200000286102295
  ],
  "speed": [
    7.199999809265137,
    14.399999618530273
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Feather")
});
