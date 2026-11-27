import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -240,
    240
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 2,
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
      0.32499998807907104,
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
  "duration": 0.016699999570846558,
  "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-0-0.effect"),
  "lifetime": [
    0.33329999446868896,
    1
  ],
  "looping": false,
  "maxParticles": 33,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    3.5999999046325684,
    3.5999999046325684
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Feather")
});
