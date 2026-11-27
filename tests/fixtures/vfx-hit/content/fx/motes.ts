import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0.6000000238418579,
        3,
        1,
        0
      ]
    ],
    [
      0.20000000298023224,
      [
        0.800000011920929,
        4,
        1.5,
        1
      ]
    ],
    [
      1,
      [
        0.30000001192092896,
        2,
        0.800000011920929,
        0
      ]
    ]
  ],
  "drag": 0.800000011920929,
  "gravity": [
    0,
    1.600000023841858,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.399999976158142
  ],
  "maxParticles": 128,
  "shape": {
    "kind": "sphere",
    "radius": 0.699999988079071,
    "surface": false
  },
  "size": [
    0.03999999910593033,
    0.09000000357627869
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      0.30000001192092896,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0.20000000298023224,
    0.6000000238418579
  ],
  "spread": 30,
  "texture": ref("base/textures/spark")
});
