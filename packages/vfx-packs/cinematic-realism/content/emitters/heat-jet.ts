import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    1,
    1,
    1,
    0
  ],
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
      0.25,
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
        0
      ]
    ]
  ],
  "distortion": 0.009999999776482582,
  "drag": 1.5,
  "duration": 2,
  "gravity": [
    0,
    0.800000011920929,
    0
  ],
  "lifetime": [
    0.5,
    0.8999999761581421
  ],
  "maxParticles": 24,
  "rate": 14,
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.05000000074505806
  },
  "size": [
    0.4000000059604645,
    0.699999988079071
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      1.7999999523162842
    ]
  ],
  "speed": [
    3,
    6
  ]
});
