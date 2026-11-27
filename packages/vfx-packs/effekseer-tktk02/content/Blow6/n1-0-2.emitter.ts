import { file } from '@origingame/engine/author';
// Effekseer node 1.0.2 "Distortion"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0781,
      0.2636
    ],
    [
      0.1719,
      0.5118
    ],
    [
      0.2656,
      0.6949
    ],
    [
      0.375,
      0.8395
    ],
    [
      0.4688,
      0.9156
    ],
    [
      0.5781,
      0.9625
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
        0.2195,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        0.1255
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.004
});
