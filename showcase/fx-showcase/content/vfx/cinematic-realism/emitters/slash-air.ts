import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1,
    1,
    1,
    0
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
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "distortion": 0.012000000104308128,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.20000000298023224,
    0.30000001192092896
  ],
  "looping": false,
  "maxParticles": 6,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.8999999761581421,
    "surface": true
  },
  "size": [
    0.5,
    0.699999988079071
  ],
  "speed": [
    0,
    0
  ]
});
