import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.03999999910593033
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 22,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.30000001192092896
  },
  "color": [
    6,
    3,
    1.100000023841858,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1.399999976158142,
        1.2000000476837158,
        1,
        1
      ]
    ],
    [
      0.5,
      [
        1,
        0.550000011920929,
        0.20000000298023224,
        1
      ]
    ],
    [
      1,
      [
        0.800000011920929,
        0.25,
        0.05000000074505806,
        0
      ]
    ]
  ],
  "drag": 1.2000000476837158,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.15000000596046448,
    0.44999998807907104
  ],
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 45,
    "radius": 0.009999999776482582
  },
  "size": [
    0.009999999776482582,
    0.017999999225139618
  ],
  "speed": [
    3,
    9
  ],
  "spread": 15
});
