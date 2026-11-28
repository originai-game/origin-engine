import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    500,
    1100
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.44999998807907104,
    "die": false,
    "friction": 0.5
  },
  "color": [
    0.800000011920929,
    0.5799999833106995,
    0.2199999988079071,
    1
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
      0.8999999761581421,
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
  "drag": 0.20000000298023224,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      3.5999999046325684
    ],
    [
      1,
      3.5999999046325684
    ]
  ],
  "lifetime": [
    2,
    2
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 1,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.3499999940395355,
  "shape": {
    "kind": "cone",
    "angleDegrees": 10,
    "radius": 0.004999999888241291
  },
  "size": [
    0.012000000104308128,
    0.012000000104308128
  ],
  "speed": [
    2.200000047683716,
    3
  ]
});
