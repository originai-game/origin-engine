import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -720,
    720
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.5
  },
  "color": [
    0.41999998688697815,
    0.3799999952316284,
    0.33000001311302185,
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
      0.800000011920929,
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
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.6000000238418579,
    1
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 35,
    "radius": 0.019999999552965164
  },
  "sheet": {
    "blend": false,
    "columns": 3,
    "frames": 3,
    "mode": {
      "kind": "random"
    },
    "rows": 1
  },
  "size": [
    0.02500000037252903,
    0.05999999865889549
  ],
  "speed": [
    2,
    5
  ],
  "spread": 10,
  "texture": ref("base/vfx/realistic-starter/textures/dirt")
});
