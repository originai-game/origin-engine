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
  "bursts": [
    {
      "count": 24,
      "cycles": 2,
      "interval": 0.4000000059604645,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0
      ]
    ],
    [
      0.1875,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.14139999449253082
      ]
    ],
    [
      0.2655999958515167,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.2328999936580658
      ]
    ],
    [
      0.33329999446868896,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.3346000015735626
      ]
    ],
    [
      1,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0
      ]
    ]
  ],
  "duration": 0.41670000553131104,
  "effect": ref("base/vfx/effekseer-starter/FireBall/n0-1-0-0-1.effect"),
  "heightOverLife": [
    [
      0,
      0.25
    ],
    [
      1,
      1.25
    ]
  ],
  "lifetime": [
    0.41670000553131104,
    0.583299994468689
  ],
  "looping": false,
  "maxParticles": 60,
  "rate": 0,
  "rotation": [
    -6,
    6
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    4,
    4
  ],
  "sizeOverLife": [
    [
      0,
      0.800000011920929
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    3.5999999046325684,
    3.5999999046325684
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Fire_Blur")
});
