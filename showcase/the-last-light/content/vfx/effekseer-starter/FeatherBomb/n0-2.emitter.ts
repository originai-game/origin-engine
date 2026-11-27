import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 13,
      "cycles": 1,
      "interval": 0,
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
      0.13330000638961792,
      [
        1,
        1,
        1,
        0.1881999969482422
      ]
    ],
    [
      0.4000000059604645,
      [
        1,
        1,
        1,
        0.1881999969482422
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
  "distortion": 0.019999999552965164,
  "duration": 0.016699999570846558,
  "lifetime": [
    0.75,
    0.75
  ],
  "looping": false,
  "maxParticles": 17,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      1,
      1
    ]
  },
  "size": [
    36,
    36
  ],
  "sizeOverLife": [
    [
      0,
      0.125
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/ExperimentalBlur")
});
