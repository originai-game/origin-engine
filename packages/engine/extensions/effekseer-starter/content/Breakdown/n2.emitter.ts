import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -120,
    120
  ],
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
        0.2508000135421753,
        0,
        0.2508000135421753,
        0.5019999742507935
      ]
    ],
    [
      0.6000000238418579,
      [
        0.2508000135421753,
        0,
        0.2508000135421753,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        0.2508000135421753,
        0,
        0.2508000135421753,
        0
      ]
    ]
  ],
  "duration": 0.25,
  "heightOverLife": [
    [
      0,
      0.0625
    ],
    [
      1,
      0.0625
    ]
  ],
  "lifetime": [
    0.05000000074505806,
    0.2833000123500824
  ],
  "looping": false,
  "maxParticles": 14,
  "randomColorOverLife": [
    [
      0,
      [
        0.9405999779701233,
        1,
        0.9405999779701233,
        0.5019999742507935
      ]
    ],
    [
      0.6000000238418579,
      [
        0.9405999779701233,
        1,
        0.9405999779701233,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        0.9405999779701233,
        1,
        0.9405999779701233,
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
    "kind": "point"
  },
  "size": [
    6,
    10
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/ShockWave3")
});
