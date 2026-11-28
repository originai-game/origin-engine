import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -900,
    900
  ],
  "blend": "alpha",
  "collision": {
    "against": "depth",
    "bounce": 0.3499999940395355,
    "die": false,
    "friction": 0.5
  },
  "color": [
    0.949999988079071,
    0.7200000286102295,
    0.20000000298023224,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.8999999761581421,
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
  "drag": 0.20000000298023224,
  "duration": 0.5,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      0.1599999964237213
    ],
    [
      1,
      0.1599999964237213
    ]
  ],
  "lifetime": [
    1.399999976158142,
    1.7999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "render": {
    "kind": "mesh",
    "mesh": "sphere"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.30000001192092896,
  "shape": {
    "kind": "cone",
    "angleDegrees": 28,
    "radius": 0.05000000074505806
  },
  "size": [
    0.03999999910593033,
    0.05000000074505806
  ],
  "speed": [
    3,
    5
  ]
});
