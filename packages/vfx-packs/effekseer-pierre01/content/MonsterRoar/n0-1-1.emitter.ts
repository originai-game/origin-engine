import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1 "Shockwave"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 9,
      "interval": 0.1667
    }
  ],
  "duration": 1.35,
  "looping": false,
  "lifetime": [
    0.4167,
    1.0833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.0,
    12.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.3281,
      0.6419
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1111,
      [
        1.0,
        1.0,
        1.0,
        0.0784
      ]
    ],
    [
      0.5556,
      [
        1.0,
        1.0,
        1.0,
        0.0784
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
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
    -12.0,
    12.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Halo"
  },
  "blend": "additive"
});
