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
  "distortion": 0.00800000037997961,
  "drag": 0.800000011920929,
  "duration": 2,
  "gravity": [
    0,
    1,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    1.7999999523162842
  ],
  "maxParticles": 8,
  "rate": 3,
  "shape": {
    "kind": "sphere",
    "radius": 0.15000000596046448,
    "surface": false
  },
  "size": [
    0.5,
    0.800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.600000023841858
    ]
  ],
  "speed": [
    0.30000001192092896,
    0.6000000238418579
  ]
});
