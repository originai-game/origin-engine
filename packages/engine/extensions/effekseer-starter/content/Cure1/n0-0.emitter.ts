import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.8592000007629395,
        0.9075000286102295,
        0.20110000669956207,
        0.7843000292778015
      ]
    ],
    [
      0.25,
      [
        0.8932999968528748,
        0.9301999807357788,
        0.3391999900341034,
        0.6373000144958496
      ]
    ],
    [
      0.5,
      [
        0.9280999898910522,
        0.9531999826431274,
        0.5174000263214111,
        0.490200012922287
      ]
    ],
    [
      0.75,
      [
        0.963699996471405,
        0.9764000177383423,
        0.7372000217437744,
        0.34310001134872437
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0.19609999656677246
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 7,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    0.75,
    0.75
  ],
  "speed": [
    2,
    2
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
