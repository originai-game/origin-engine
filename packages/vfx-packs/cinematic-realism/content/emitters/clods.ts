import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -400,
    400
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 14,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.10000000149011612,
    "die": false,
    "friction": 0.8999999761581421
  },
  "color": [
    0.20000000298023224,
    0.15000000596046448,
    0.10000000149011612,
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
  "drag": 0.4000000059604645,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.2999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 35,
    "radius": 0.03999999910593033
  },
  "size": [
    0.009999999776482582,
    0.02800000086426735
  ],
  "speed": [
    2,
    5
  ]
});
