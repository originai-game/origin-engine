import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3 "Flash"
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
      "count": 1,
      "cycles": 3,
      "interval": 0.0417
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.3,
    0.3
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    36.0,
    36.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.25
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.1166,
        0.0254,
        0.1394
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.1166,
        0.0254,
        0.2689
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
