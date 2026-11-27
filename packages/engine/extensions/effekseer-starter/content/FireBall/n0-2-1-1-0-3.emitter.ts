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
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0
      ]
    ],
    [
      0.16670000553131104,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.04820000007748604
      ]
    ],
    [
      0.2777999937534332,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0.1088000014424324
      ]
    ],
    [
      1,
      [
        1,
        0.11659999936819077,
        0.010400000028312206,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "effect": ref("base/vfx/effekseer-starter/FireBall/n0-2-1-1-0-3.effect"),
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
    0.516700029373169,
    0.6833000183105469
  ],
  "looping": false,
  "maxParticles": 68,
  "rate": 0,
  "rotation": [
    -6,
    6
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.8999999761581421,
      1.5,
      0.8999999761581421
    ]
  },
  "size": [
    9.5,
    9.5
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
    45.98979949951172,
    70.50509643554688
  ],
  "speedOverLife": [
    [
      0,
      1.305400013923645
    ],
    [
      0.5156000256538391,
      0.9740999937057495
    ],
    [
      1,
      0.7627999782562256
    ]
  ],
  "spread": 18.156099319458008,
  "texture": ref("base/vfx/effekseer-starter/textures/Fire_Blur")
});
