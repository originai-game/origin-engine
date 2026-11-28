import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -700,
    700
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 12,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.6000000238418579
  },
  "color": [
    0.5,
    0.49000000953674316,
    0.46000000834465027,
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
    0.699999988079071,
    1.2999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 14,
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
    "angleDegrees": 45,
    "radius": 0.029999999329447746
  },
  "size": [
    0.00800000037997961,
    0.02500000037252903
  ],
  "speed": [
    1.5,
    4.5
  ]
});
