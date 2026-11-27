import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "color": [
    1.399999976158142,
    1.340000033378601,
    1.2200000286102295,
    1
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
        0.12999999523162842
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.08500000089406967
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
  "drag": 1.600000023841858,
  "duration": 2,
  "gravity": [
    0,
    0.019999999552965164,
    0
  ],
  "lifetime": [
    1.600000023841858,
    2.5999999046325684
  ],
  "lit": true,
  "maxParticles": 400,
  "rate": 150,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 89,
    "radius": 1.7999999523162842
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    1,
    1.5
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.8999999761581421,
  "speed": [
    4,
    7
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
