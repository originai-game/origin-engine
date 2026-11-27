import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 15,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.010400000028312206,
        0,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        0.010400000028312206,
        0,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.010400000028312206,
        0,
        0
      ]
    ]
  ],
  "duration": 0.25,
  "lifetime": [
    0.08330000191926956,
    0.25
  ],
  "looping": false,
  "maxParticles": 14,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        0.11659999936819077,
        0.04780000075697899,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        0.11659999936819077,
        0.04780000075697899,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.11659999936819077,
        0.04780000075697899,
        0
      ]
    ]
  ],
  "rate": 0,
  "rotation": [
    -180,
    180
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "size": [
    0.44999998807907104,
    0.949999988079071
  ],
  "speed": [
    6.495200157165527,
    25.98080062866211
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/particle2")
});
