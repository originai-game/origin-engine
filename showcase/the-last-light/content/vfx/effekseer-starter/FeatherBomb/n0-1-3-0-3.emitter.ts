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
  "blend": "alpha",
  "bursts": [
    {
      "count": 26,
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
        1,
        1,
        0
      ]
    ],
    [
      0.09380000084638596,
      [
        1,
        1,
        1,
        0.04320000112056732
      ]
    ],
    [
      0.16670000553131104,
      [
        1,
        1,
        1,
        0.09640000015497208
      ]
    ],
    [
      0.2777999937534332,
      [
        1,
        1,
        1,
        0.2175000011920929
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
  "duration": 0.016699999570846558,
  "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-3.effect"),
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
  "maxParticles": 65,
  "rate": 0,
  "rotation": [
    -6,
    6
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.675000011920929,
      1.125,
      0.675000011920929
    ]
  },
  "size": [
    7.125,
    7.125
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
    34.4922981262207,
    52.87879943847656
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
