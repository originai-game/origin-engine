import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "火尻尾"
export default file.particles({
  "maxParticles": 2194,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.25,
    0.9167
  ],
  "speed": [
    2.2269,
    8.9077
  ],
  "size": [
    3.0,
    3.0
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
        0.8832,
        0.0179,
        0.0179,
        1.0
      ]
    ],
    [
      0.2656,
      [
        0.8875,
        0.0742,
        0.0164,
        0.7344
      ]
    ],
    [
      0.5156,
      [
        0.8915,
        0.1675,
        0.0151,
        0.4844
      ]
    ],
    [
      0.7656,
      [
        0.8956,
        0.303,
        0.0138,
        0.2344
      ]
    ],
    [
      1.0,
      [
        0.8994,
        0.4704,
        0.0127,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
