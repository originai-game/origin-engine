import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "破裂"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 8,
      "interval": 0.0333
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1667
    ],
    [
      0.0781,
      0.3864
    ],
    [
      0.1719,
      0.5932
    ],
    [
      0.2656,
      0.7458
    ],
    [
      0.375,
      0.8663
    ],
    [
      0.4688,
      0.9297
    ],
    [
      0.5781,
      0.9687
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
        0.2195,
        1.0,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Wind"
  },
  "blend": "additive"
});
