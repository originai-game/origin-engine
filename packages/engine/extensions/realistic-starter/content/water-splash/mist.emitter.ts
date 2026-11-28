import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0.05000000074505806
    }
  ],
  "color": [
    0.8799999952316284,
    0.9200000166893005,
    0.9599999785423279,
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
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.2800000011920929
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
  "drag": 2,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.30000001192092896,
    "surface": false
  },
  "sheet": {
    "blend": true,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 2
  },
  "size": [
    0.4000000059604645,
    0.6499999761581421
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.899999976158142
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.30000001192092896,
    0.8999999761581421
  ],
  "spread": 180,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
