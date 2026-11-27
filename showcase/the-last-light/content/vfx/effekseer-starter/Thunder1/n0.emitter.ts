import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 40,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.5356000065803528,
        0,
        0
      ]
    ],
    [
      0.02500000037252903,
      [
        1,
        0.5356000065803528,
        0,
        1
      ]
    ],
    [
      0.875,
      [
        1,
        0.5356000065803528,
        0,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.5356000065803528,
        0,
        0
      ]
    ]
  ],
  "duration": 0.666700005531311,
  "gravity": [
    0,
    0.7200000286102295,
    0
  ],
  "lifetime": [
    1.166700005531311,
    1.5
  ],
  "looping": false,
  "maxParticles": 50,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        1,
        0,
        0
      ]
    ],
    [
      0.02500000037252903,
      [
        1,
        1,
        0,
        1
      ]
    ],
    [
      0.875,
      [
        1,
        1,
        0,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        0,
        0
      ]
    ]
  ],
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.2000000476837158,
      0.5,
      1.2000000476837158
    ]
  },
  "size": [
    1,
    1
  ],
  "speed": [
    0.6000000238418579,
    1.7999999523162842
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/particle4")
});
