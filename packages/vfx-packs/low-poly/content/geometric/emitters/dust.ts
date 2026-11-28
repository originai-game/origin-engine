import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -25,
    25
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.25,
    0.3199999928474426,
    0.3799999952316284,
    1
  ],
  "drag": 1.7999999523162842,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.05999999865889549,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.399999976158142
  ],
  "looping": false,
  "maxParticles": 12,
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
    "angleDegrees": 85,
    "radius": 0.10000000149011612
  },
  "size": [
    0.07999999821186066,
    0.1599999964237213
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      0.25,
      1
    ],
    [
      0.6000000238418579,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0.4000000059604645,
    0.800000011920929
  ]
});
