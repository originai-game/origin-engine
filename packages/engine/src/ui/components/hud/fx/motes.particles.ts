import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "duration": 1.399999976158142,
  "gravity": [
    0,
    40,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    2
  ],
  "looping": false,
  "maxParticles": 48,
  "rate": 20,
  "shape": {
    "kind": "sphere",
    "radius": 110,
    "surface": false
  },
  "size": [
    6,
    12
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.20000000298023224,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    10,
    40
  ],
  "spread": 180,
  "texture": ref("@styles/shared/fx/glint.png")
});
