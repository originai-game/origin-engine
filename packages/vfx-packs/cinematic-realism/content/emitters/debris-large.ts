import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -400,
    400
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 40,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.25,
    "die": false,
    "friction": 0.699999988079071
  },
  "color": [
    0.09000000357627869,
    0.07999999821186066,
    0.07000000029802322,
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
  "lifetime": [
    1.7999999523162842,
    3
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 44,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.8999999761581421,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.800000011920929
  },
  "size": [
    0.05000000074505806,
    0.14000000059604645
  ],
  "speed": [
    8,
    16
  ]
});
