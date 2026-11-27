import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -540,
    540
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 16,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.25,
    "die": false,
    "friction": 0.6000000238418579
  },
  "color": [
    0.3199999928474426,
    0.2800000011920929,
    0.23999999463558197,
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
      0.8500000238418579,
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
    1.2999999523162842,
    1.899999976158142
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.30000001192092896
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
    0.05999999865889549,
    0.15000000596046448
  ],
  "speed": [
    4,
    9
  ],
  "spread": 15,
  "texture": ref("base/vfx/realistic-starter/textures/dirt")
});
