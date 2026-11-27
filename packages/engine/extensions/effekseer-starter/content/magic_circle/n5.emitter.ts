import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
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
        1
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.4611999988555908,
    0.4611999988555908
  ],
  "sizeOverLife": [
    [
      0,
      0.8235999941825867
    ],
    [
      0.4530999958515167,
      1.0037000179290771
    ],
    [
      1,
      0.765999972820282
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/eggzero")
});
