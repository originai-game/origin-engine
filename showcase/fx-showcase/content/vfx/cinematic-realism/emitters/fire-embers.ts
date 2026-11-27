import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        2.4000000953674316,
        1.399999976158142,
        0.4000000059604645,
        1
      ]
    ],
    [
      0.5,
      [
        1.7999999523162842,
        0.6000000238418579,
        0.10000000149011612,
        1
      ]
    ],
    [
      1,
      [
        0.4000000059604645,
        0.05000000074505806,
        0.009999999776482582,
        0
      ]
    ]
  ],
  "drag": 0.6000000238418579,
  "duration": 2,
  "gravity": [
    0,
    0.6000000238418579,
    0
  ],
  "lifetime": [
    1,
    2.200000047683716
  ],
  "maxParticles": 32,
  "noise": {
    "frequency": 1,
    "scroll": 0.4000000059604645,
    "strength": 0.5
  },
  "rate": 9,
  "shape": {
    "kind": "cone",
    "angleDegrees": 20,
    "radius": 0.15000000596046448
  },
  "size": [
    0.00800000037997961,
    0.01600000075995922
  ],
  "speed": [
    0.5,
    1.5
  ],
  "wind": 1
});
