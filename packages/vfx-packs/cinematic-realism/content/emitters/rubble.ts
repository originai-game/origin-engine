import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -200,
    200
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.20000000298023224,
    "die": false,
    "friction": 0.800000011920929
  },
  "color": [
    0.44999998807907104,
    0.4399999976158142,
    0.41999998688697815,
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
      0.550000011920929
    ],
    [
      1,
      0.550000011920929
    ]
  ],
  "lifetime": [
    2,
    3
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 20,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.949999988079071,
  "shape": {
    "kind": "sphere",
    "radius": 0.4000000059604645,
    "surface": false
  },
  "size": [
    0.07000000029802322,
    0.2199999988079071
  ],
  "speed": [
    1.5,
    5
  ],
  "spread": 120
});
