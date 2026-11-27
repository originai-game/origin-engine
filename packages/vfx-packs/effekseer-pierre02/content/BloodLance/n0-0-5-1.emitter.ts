import { file } from '@origingame/engine/author';
// Effekseer node 0.0.5.1 "Flash"
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
    0.35,
    0.35
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    64.0,
    64.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
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
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.1429,
      [
        1.0,
        0.1166,
        0.0254,
        0.251
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.1166,
        0.0254,
        0.251
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
