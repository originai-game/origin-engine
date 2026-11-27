import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "炎上がる"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 17.4464,
  "rate": 3000.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    8.3858,
    17.075
  ],
  "size": [
    2.6667,
    2.6667
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.9828,
        0.9158,
        0.1113,
        0.7843
      ]
    ],
    [
      0.2344,
      [
        0.9868,
        0.589,
        0.0889,
        0.6005
      ]
    ],
    [
      0.3594,
      [
        0.989,
        0.4463,
        0.078,
        0.5025
      ]
    ],
    [
      0.4844,
      [
        0.9911,
        0.3249,
        0.0679,
        0.4044
      ]
    ],
    [
      0.7344,
      [
        0.9954,
        0.1434,
        0.05,
        0.2083
      ]
    ],
    [
      0.8594,
      [
        0.9976,
        0.0818,
        0.0422,
        0.1103
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0342,
        0.0342,
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
