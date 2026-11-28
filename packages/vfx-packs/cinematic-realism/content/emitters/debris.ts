import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -540,
    540
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 26,
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
      0.8500000238418579,
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
  "drag": 0.30000001192092896,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    1,
    1.7999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 32,
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
    "angleDegrees": 55,
    "radius": 0.30000001192092896
  },
  "size": [
    0.02500000037252903,
    0.05999999865889549
  ],
  "speed": [
    3.5,
    7
  ]
});
