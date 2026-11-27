import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "alpha",
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 0
  },
  "color": [
    0.8799999952316284,
    0.9399999976158142,
    1,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.800000011920929
      ]
    ],
    [
      0.8999999761581421,
      [
        1,
        1,
        1,
        0.6000000238418579
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
  "drag": 0.10000000149011612,
  "duration": 2,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    1.399999976158142,
    1.399999976158142
  ],
  "maxParticles": 400,
  "rate": 240,
  "shape": {
    "kind": "cone",
    "angleDegrees": 4,
    "radius": 0.019999999552965164
  },
  "size": [
    0.019999999552965164,
    0.03500000014901161
  ],
  "speed": [
    5.5,
    6.199999809265137
  ]
});
