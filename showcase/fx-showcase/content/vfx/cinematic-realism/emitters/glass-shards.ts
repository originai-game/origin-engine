import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -800,
    800
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 20,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.4000000059604645
  },
  "color": [
    0.7799999713897705,
    0.8799999952316284,
    0.9200000166893005,
    0.550000011920929
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
    0.800000011920929,
    1.399999976158142
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.05999999865889549,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.03999999910593033
  },
  "size": [
    0.009999999776482582,
    0.029999999329447746
  ],
  "speed": [
    1.5,
    5
  ]
});
