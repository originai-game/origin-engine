import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.26669999957084656,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.733299970626831,
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
  "duration": 0.41670000553131104,
  "lifetime": [
    1.25,
    1.25
  ],
  "looping": false,
  "maxParticles": 125,
  "rate": 240,
  "rotation": [
    -180,
    180
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    1,
    1
  ],
  "speed": [
    1.7999999523162842,
    3
  ],
  "speedOverLife": [
    [
      0,
      3.189500093460083
    ],
    [
      0.15620000660419464,
      2.224100112915039
    ],
    [
      0.28119999170303345,
      1.57669997215271
    ],
    [
      0.46880000829696655,
      0.8501999974250793
    ],
    [
      0.6561999917030334,
      0.4174000024795532
    ],
    [
      0.6718999743461609,
      0.06729999929666519
    ],
    [
      0.6875,
      0
    ],
    [
      1,
      0
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/fog1")
});
