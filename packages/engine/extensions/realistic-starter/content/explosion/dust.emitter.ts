import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -25,
    25
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0.029999999329447746
    }
  ],
  "color": [
    0.7200000286102295,
    0.6399999856948853,
    0.5400000214576721,
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
      0.10000000149011612,
      [
        1,
        1,
        1,
        0.699999988079071
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
  "drag": 2.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.15000000596046448,
    0
  ],
  "lifetime": [
    1.399999976158142,
    2.200000047683716
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
    "angleDegrees": 84,
    "radius": 0.4000000059604645
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
    0.6000000238418579,
    1
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2
    ]
  ],
  "softDistance": 0.5,
  "speed": [
    3,
    5.5
  ],
  "spread": 4,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
