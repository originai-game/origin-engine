import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -600,
    600
  ],
  "blend": "alpha",
  "collision": {
    "against": "depth",
    "bounce": 0.20000000298023224,
    "die": true,
    "friction": 0.699999988079071
  },
  "color": [
    0.20000000298023224,
    0.18000000715255737,
    0.15000000596046448,
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
  "duration": 2,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.6000000238418579,
    1.100000023841858
  ],
  "lit": true,
  "maxParticles": 40,
  "rate": 18,
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
    "radius": 0.10000000149011612
  },
  "size": [
    0.00800000037997961,
    0.019999999552965164
  ],
  "speed": [
    2,
    4.5
  ]
});
