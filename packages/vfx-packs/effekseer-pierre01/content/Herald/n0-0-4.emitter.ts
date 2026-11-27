import { file } from '@origingame/engine/author';
// Effekseer node 0.0.4 "Lightwave"
export default file.particles({
  "maxParticles": 6,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 13,
      "interval": 0.1
    }
  ],
  "duration": 1.2167,
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
    3.4,
    3.4
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7059
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
        1.0,
        0.3587,
        0.8751,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.8065,
        0.3204,
        0.9055,
        0.1255
      ]
    ],
    [
      0.5,
      [
        0.6354,
        0.2845,
        0.9364,
        0.251
      ]
    ],
    [
      0.75,
      [
        0.4863,
        0.2508,
        0.9679,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Halo"
  },
  "blend": "additive"
});
