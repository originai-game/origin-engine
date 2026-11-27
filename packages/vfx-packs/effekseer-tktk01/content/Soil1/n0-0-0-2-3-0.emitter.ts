import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.3.0 "消滅時線"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    6.0,
    12.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/rainntya"
  },
  "blend": "additive"
});
