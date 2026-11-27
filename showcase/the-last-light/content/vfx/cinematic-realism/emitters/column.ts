import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0.18000000715255737
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 0
  },
  "color": [
    0.8600000143051147,
    0.9300000071525574,
    1,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.8999999761581421
      ]
    ],
    [
      0.75,
      [
        1,
        1,
        1,
        0.699999988079071
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
  "drag": 0.20000000298023224,
  "duration": 0.25,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.44999998807907104,
    0.800000011920929
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 4,
    "radius": 0.029999999329447746
  },
  "size": [
    0.03999999910593033,
    0.07500000298023224
  ],
  "speed": [
    1.7999999523162842,
    3
  ]
});
