import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      0,
      1
    ]
  },
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        0,
        0,
        0.5019999742507935
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        0,
        0,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        1,
        0,
        0,
        0
      ]
    ]
  ],
  "duration": 0.25,
  "heightOverLife": [
    [
      0,
      2
    ],
    [
      1,
      2
    ]
  ],
  "lifetime": [
    0.16670000553131104,
    0.16670000553131104
  ],
  "looping": false,
  "maxParticles": 27,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        1,
        0.04780000075697899,
        0.5019999742507935
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        0.04780000075697899,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        1,
        1,
        0.04780000075697899,
        0
      ]
    ]
  ],
  "rate": 120,
  "shape": {
    "kind": "sphere",
    "radius": 1.5,
    "surface": false
  },
  "size": [
    2,
    2
  ],
  "speed": [
    30,
    30
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Line6")
});
