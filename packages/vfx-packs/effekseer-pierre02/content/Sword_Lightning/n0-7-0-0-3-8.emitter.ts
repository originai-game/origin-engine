import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.8 "Lightwave"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4,
    0.4
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    80.0,
    80.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.75
    ],
    [
      0.1562,
      0.8209
    ],
    [
      0.3281,
      0.8744
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
        0.586,
        0.5295,
        1.0,
        0.3012
      ]
    ],
    [
      1.0,
      [
        0.586,
        0.5295,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -15.0,
    15.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Halo"
  },
  "blend": "additive"
});
