import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
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
        0,
        0.5356000065803528,
        0.6274999976158142
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        0,
        0.5356000065803528,
        0.6274999976158142
      ]
    ],
    [
      1,
      [
        1,
        0,
        0.5356000065803528,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.25,
    0.25
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    6,
    6
  ],
  "sizeOverLife": [
    [
      0,
      0.33329999446868896
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
  "texture": ref("base/vfx/effekseer-starter/textures/particle")
});
