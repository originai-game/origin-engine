import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.9.0.0 "SparkLight"
export default file.particles({
  "maxParticles": 350,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
      "cycles": 60,
      "interval": 0.0167
    }
  ],
  "duration": 1.0,
  "looping": false,
  "lifetime": [
    0.1,
    0.1
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.7
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        0.0478,
        1.0,
        0.0627
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        0.0478,
        1.0,
        0.1004
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
