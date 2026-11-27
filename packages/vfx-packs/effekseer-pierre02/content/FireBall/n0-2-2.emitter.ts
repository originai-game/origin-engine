import { file } from '@origingame/engine/author';
// Effekseer node 0.2.2 "Flash"
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 9
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
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
      0.1667
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        0.2195,
        0.0478,
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2195,
        0.0478,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
