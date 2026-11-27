import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 9,
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
      0.2678999900817871,
      [
        1,
        1,
        1,
        0.3765000104904175
      ]
    ],
    [
      0.4643000066280365,
      [
        1,
        1,
        1,
        0.3765000104904175
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
    1.4666999578475952,
    2.266700029373169
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.25,
      0.25
    ]
  },
  "size": [
    27,
    27
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0.666700005531311
    ]
  ],
  "speed": [
    1.7999999523162842,
    1.7999999523162842
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/ExperimentalNoise")
});
