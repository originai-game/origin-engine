import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -1200,
    1200
  ],
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.0333000011742115,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        1
      ]
    ]
  ],
  "duration": 0.33329999446868896,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 300,
  "rate": 720,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.25,
    0.75
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/star")
});
