import { file } from '@origingame/engine/author';
// Effekseer node 0.4 "Twinkle Faster"
export default file.particles({
  "maxParticles": 127,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    8.4853,
    33.9411
  ],
  "size": [
    0.75,
    2.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.2195,
        0.5356,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.2195,
        0.5356,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2195,
        0.5356,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2195,
        0.5356,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
