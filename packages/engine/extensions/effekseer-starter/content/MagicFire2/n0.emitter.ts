import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
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
      1,
      [
        1,
        1,
        1,
        1
      ]
    ]
  ],
  "duration": 1.583299994468689,
  "gravity": [
    10.800000190734863,
    0,
    0
  ],
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 52,
  "rate": 120,
  "shape": {
    "kind": "point"
  },
  "size": [
    2.0299999713897705,
    2.0299999713897705
  ],
  "sizeOverLife": [
    [
      0,
      0.4925999939441681
    ],
    [
      0.25,
      0.5640000104904175
    ],
    [
      0.5,
      0.6723999977111816
    ],
    [
      0.75,
      0.8177000284194946
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    10.783300399780273,
    14.236599922180176
  ],
  "spread": 18.931800842285156,
  "texture": ref("base/vfx/effekseer-starter/textures/FlameRadiation")
});
