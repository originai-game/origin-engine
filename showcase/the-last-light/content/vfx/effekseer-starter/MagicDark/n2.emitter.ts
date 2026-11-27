import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "subtractive",
  "bursts": [
    {
      "count": 1,
      "cycles": 20,
      "interval": 0.016699999570846558,
      "time": 0
    }
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
      0.125,
      [
        1,
        1,
        1,
        0.2736999988555908
      ]
    ],
    [
      0.25,
      [
        1,
        1,
        1,
        0.482699990272522
      ]
    ],
    [
      0.6718999743461609,
      [
        1,
        1,
        1,
        0.24369999766349792
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
  "duration": 0.33329999446868896,
  "heightOverLife": [
    [
      0,
      0.33329999446868896
    ],
    [
      1,
      0.666700005531311
    ]
  ],
  "lifetime": [
    0.666700005531311,
    0.666700005531311
  ],
  "looping": false,
  "maxParticles": 25,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      2,
      1
    ]
  },
  "size": [
    3,
    3
  ],
  "sizeOverLife": [
    [
      0,
      0.33329999446868896
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    2.1865999698638916,
    8.746399879455566
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/Fog3001")
});
